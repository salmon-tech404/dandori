//! Bộ thực thi tiến trình con an toàn với timeout và quản lý cây tiến trình.

use crate::error::ProcessError;
use crate::event::ProcessEvent;
use crate::spec::ProcessSpec;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc::{Sender, channel};
use tokio::time::{Instant, sleep};
use tracing::warn;

const CHANNEL_BUFFER: usize = 128;

/// Khởi chạy tiến trình con theo đặc tả an toàn.
pub fn spawn(spec: ProcessSpec) -> Result<tokio::sync::mpsc::Receiver<ProcessEvent>, ProcessError> {
    let mut cmd = build_command(&spec)?;
    let child = cmd.spawn()?;
    let pid = child.id().ok_or(ProcessError::MissingPid)?;

    let (tx, rx) = channel::<ProcessEvent>(CHANNEL_BUFFER);
    tokio::spawn(run_process_loop(child, pid, spec, tx));

    Ok(rx)
}

fn build_command(spec: &ProcessSpec) -> Result<tokio::process::Command, ProcessError> {
    #[expect(
        clippy::disallowed_methods,
        reason = "dandori-process là nơi duy nhất được tạo tiến trình con"
    )]
    let mut cmd = tokio::process::Command::new(&spec.program);
    cmd.args(&spec.args);
    cmd.current_dir(&spec.working_dir);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.kill_on_drop(true);

    // `process_group` là hàm riêng của tokio::process::Command (không cần trait CommandExt của std).
    #[cfg(unix)]
    cmd.process_group(0);

    setup_environment(&mut cmd, spec);
    Ok(cmd)
}

fn setup_environment(cmd: &mut tokio::process::Command, spec: &ProcessSpec) {
    cmd.env_clear();
    for var in &spec.env_whitelist {
        if let Ok(val) = std::env::var(var) {
            cmd.env(var, val);
        }
    }
    for (k, v) in &spec.custom_env {
        cmd.env(k, v);
    }
    if let Some(ref home) = spec.isolated_home {
        cmd.env("HOME", home);
        #[cfg(windows)]
        {
            cmd.env("USERPROFILE", home);
            cmd.env("APPDATA", home.join("AppData").join("Roaming"));
            cmd.env("LOCALAPPDATA", home.join("AppData").join("Local"));
        }
    }
}

async fn run_process_loop(
    mut child: tokio::process::Child,
    pid: u32,
    spec: ProcessSpec,
    tx: Sender<ProcessEvent>,
) {
    let stdout_reader = child.stdout.take().map(|s| BufReader::new(s).lines());
    let stderr_reader = child.stderr.take().map(|s| BufReader::new(s).lines());

    let cancellation_token = spec.cancellation_token.clone();
    let total_deadline = spec.total_timeout.map(|d| Instant::now() + d);

    let mut state = LoopState {
        total_lines: 0,
        total_bytes: 0,
        line_limit: spec.line_limit,
        byte_limit: spec.byte_limit,
        silent_timeout: spec.silent_timeout,
        last_activity: Instant::now(),
    };

    let exit_code = monitor_streams(
        &mut child,
        pid,
        stdout_reader,
        stderr_reader,
        &mut state,
        total_deadline,
        cancellation_token,
        &tx,
    )
    .await;

    if let Some(code) = exit_code {
        let _ = tx.send(ProcessEvent::Exit(code)).await;
    }
}

struct LoopState {
    total_lines: usize,
    total_bytes: usize,
    line_limit: Option<usize>,
    byte_limit: Option<usize>,
    silent_timeout: Option<Duration>,
    last_activity: Instant,
}

#[expect(
    clippy::too_many_arguments,
    reason = "hàm điều phối nội bộ cần nhiều luồng stream và trạng thái"
)]
async fn monitor_streams(
    child: &mut tokio::process::Child,
    pid: u32,
    mut stdout: Option<tokio::io::Lines<BufReader<tokio::process::ChildStdout>>>,
    mut stderr: Option<tokio::io::Lines<BufReader<tokio::process::ChildStderr>>>,
    state: &mut LoopState,
    total_deadline: Option<Instant>,
    cancel_token: Option<tokio_util::sync::CancellationToken>,
    tx: &Sender<ProcessEvent>,
) -> Option<i32> {
    loop {
        if check_timeouts_and_cancel(child, pid, state, total_deadline, cancel_token.as_ref()).await
        {
            return None;
        }

        tokio::select! {
            biased;
            line_res = async {
                match stdout.as_mut() {
                    Some(r) => r.next_line().await,
                    None => std::future::pending().await,
                }
            } => {
                if !handle_line(line_res, true, state, tx, child, pid).await {
                    stdout = None;
                }
            }
            err_res = async {
                match stderr.as_mut() {
                    Some(r) => r.next_line().await,
                    None => std::future::pending().await,
                }
            } => {
                if !handle_line(err_res, false, state, tx, child, pid).await {
                    stderr = None;
                }
            }
            status_res = child.wait() => {
                return status_res.ok().and_then(|s| s.code()).or(Some(-1));
            }
            _ = sleep(Duration::from_millis(50)) => {}
        }
    }
}

async fn check_timeouts_and_cancel(
    child: &mut tokio::process::Child,
    pid: u32,
    state: &LoopState,
    total_deadline: Option<Instant>,
    cancel_token: Option<&tokio_util::sync::CancellationToken>,
) -> bool {
    if let Some(token) = cancel_token
        && token.is_cancelled()
    {
        kill_process_tree(pid, child).await;
        return true;
    }
    if let Some(deadline) = total_deadline
        && Instant::now() >= deadline
    {
        kill_process_tree(pid, child).await;
        return true;
    }
    if let Some(silent_dur) = state.silent_timeout
        && state.last_activity.elapsed() >= silent_dur
    {
        kill_process_tree(pid, child).await;
        return true;
    }
    false
}

async fn handle_line(
    res: Result<Option<String>, std::io::Error>,
    is_stdout: bool,
    state: &mut LoopState,
    tx: &Sender<ProcessEvent>,
    child: &mut tokio::process::Child,
    pid: u32,
) -> bool {
    match res {
        Ok(Some(line)) => {
            state.last_activity = Instant::now();
            state.total_lines += 1;
            state.total_bytes += line.len();

            let exceeded_line = state.line_limit.is_some_and(|l| state.total_lines > l);
            let exceeded_byte = state.byte_limit.is_some_and(|b| state.total_bytes > b);

            if exceeded_line || exceeded_byte {
                kill_process_tree(pid, child).await;
                return false;
            }

            let ev = if is_stdout {
                ProcessEvent::StdoutLine(line)
            } else {
                ProcessEvent::StderrLine(line)
            };
            let _ = tx.send(ev).await;
            true
        }
        _ => false,
    }
}

async fn kill_process_tree(pid: u32, child: &mut tokio::process::Child) {
    #[cfg(windows)]
    {
        #[expect(
            clippy::disallowed_methods,
            reason = "dandori-process gọi taskkill để tiêu diệt toàn bộ cây tiến trình trên Windows"
        )]
        let mut kill_cmd = tokio::process::Command::new("taskkill");
        kill_cmd.args(["/T", "/F", "/PID", &pid.to_string()]);
        if let Err(err) = kill_cmd.output().await {
            warn!(pid, ?err, "Không thể chạy taskkill trên Windows");
        }
    }

    #[cfg(unix)]
    {
        #[expect(
            clippy::disallowed_methods,
            reason = "dandori-process gọi lệnh kill của hệ thống để gửi SIGKILL tới nhóm tiến trình trên Unix"
        )]
        let mut kill_cmd = tokio::process::Command::new("kill");
        kill_cmd.args(["-KILL", "--", &format!("-{pid}")]);
        if let Err(err) = kill_cmd.output().await {
            warn!(pid, ?err, "Không thể chạy kill trên Unix");
        }
    }

    let _ = child.kill().await;
}
