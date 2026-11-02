//! Binary giả lập CLI agent chỉ dùng cho môi trường kiểm thử.

use serde::Deserialize;
use std::io::Write;

#[derive(Debug, Default, Deserialize)]
struct Scenario {
    #[serde(default)]
    print_lines: Vec<String>,
    #[serde(default)]
    sleep_before_ms: Option<u64>,
    #[serde(default)]
    sleep_after_ms: Option<u64>,
    #[serde(default)]
    exit_code: Option<i32>,
    #[serde(default)]
    spawn_grandchild: bool,
    #[serde(default)]
    grandchild_sleep_ms: Option<u64>,
    #[serde(default)]
    hang: bool,
    #[serde(default)]
    flood_lines: Option<usize>,
    #[serde(default)]
    quota_exhausted: bool,
    #[serde(default)]
    write_file: Option<WriteFileSpec>,
    #[serde(default)]
    write_pid_file: Option<String>,
    #[serde(default)]
    check_env: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WriteFileSpec {
    path: String,
    content: String,
}

fn main() {
    let scenario = load_scenario();
    execute_scenario(scenario);
}

fn load_scenario() -> Scenario {
    let args: Vec<String> = std::env::args().collect();
    let mut raw_json: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        if args.get(i).map(String::as_str) == Some("--scenario")
            && let Some(next) = args.get(i + 1)
        {
            raw_json = Some(next.clone());
            break;
        }
        i += 1;
    }

    if raw_json.is_none() {
        raw_json = std::env::var("FAKE_AGENT_SCENARIO").ok();
    }

    if raw_json.is_none() {
        for arg in args.iter().skip(1) {
            if arg.contains('{') && arg.contains('}') {
                raw_json = Some(arg.clone());
                break;
            }
        }
    }

    if let Some(ref raw) = raw_json {
        if let Ok(scen) = serde_json::from_str::<Scenario>(raw) {
            return scen;
        }
        if let Some(start) = raw.find('{')
            && let Some(end) = raw.rfind('}')
            && end > start
            && let Ok(scen) = serde_json::from_str::<Scenario>(&raw[start..=end])
        {
            return scen;
        }
    }

    raw_json
        .and_then(|json| serde_json::from_str::<Scenario>(&json).ok())
        .unwrap_or_default()
}

fn execute_scenario(scenario: Scenario) {
    if let Some(ref pid_file) = scenario.write_pid_file {
        let _ = std::fs::write(pid_file, std::process::id().to_string());
    }

    if let Some(ms) = scenario.sleep_before_ms {
        sleep_ms(ms);
    }

    if scenario.spawn_grandchild {
        spawn_grandchild_process(scenario.grandchild_sleep_ms.unwrap_or(60_000));
    }

    if let Some(spec) = scenario.write_file {
        let _ = std::fs::write(&spec.path, &spec.content);
    }

    let mut stdout = std::io::stdout().lock();
    for line in scenario.print_lines {
        let _ = writeln!(stdout, "{line}");
    }

    if let Some(ref env_name) = scenario.check_env {
        match std::env::var(env_name) {
            Ok(val) => {
                let _ = writeln!(stdout, "ENV_CHECK:{val}");
            }
            Err(_) => {
                let _ = writeln!(stdout, "ENV_CHECK:NOT_FOUND");
            }
        }
    }

    if scenario.quota_exhausted {
        let _ = writeln!(
            stdout,
            "{{\"type\":\"quota_exhausted\",\"message\":\"hết hạn mức tài khoản\"}}"
        );
    }

    if let Some(count) = scenario.flood_lines {
        for idx in 0..count {
            let _ = writeln!(stdout, "FLOOD_LINE_{idx}");
        }
    }
    let _ = stdout.flush();

    if let Some(ms) = scenario.sleep_after_ms {
        sleep_ms(ms);
    }

    if scenario.hang {
        loop {
            sleep_ms(10_000);
        }
    }

    if let Some(code) = scenario.exit_code {
        std::process::exit(code);
    }
}

fn sleep_ms(ms: u64) {
    #[expect(
        clippy::disallowed_methods,
        reason = "fake-agent binary dùng thread::sleep cho kịch bản"
    )]
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

fn spawn_grandchild_process(sleep_duration_ms: u64) {
    if let Ok(exe) = std::env::current_exe() {
        let scenario_json = format!(
            "{{\"sleep_before_ms\":{sleep_duration_ms},\"write_pid_file\":\"grandchild.pid\"}}"
        );
        #[expect(
            clippy::disallowed_methods,
            reason = "fake-agent spawn tiến trình cháu để test kill tree"
        )]
        let _ = std::process::Command::new(exe)
            .arg("--scenario")
            .arg(scenario_json)
            .spawn();
    }
}
