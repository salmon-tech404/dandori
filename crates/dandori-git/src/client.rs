//! Hiện thực GitClient bọc các lệnh git qua `dandori-process`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use dandori_process::{ProcessEvent, ProcessSpec, spawn};

use crate::error::GitError;
use crate::info::WorktreeInfo;
use crate::sanitize::validate_ref_name;

const DEFAULT_GIT_TIMEOUT: Duration = Duration::from_secs(30);

/// Đối tượng thực thi các thao tác Git an toàn trên một repository.
#[derive(Debug, Clone)]
pub struct GitClient {
    /// Đường dẫn thư mục kho git (chứa `.git`).
    pub repo_path: PathBuf,
}

impl GitClient {
    /// Khởi tạo `GitClient` cho một repository.
    #[must_use]
    pub fn new(repo_path: impl Into<PathBuf>) -> Self {
        Self {
            repo_path: repo_path.into(),
        }
    }

    /// Chạy lệnh git với môi trường cô lập an toàn.
    async fn run_git(
        &self,
        working_dir: &Path,
        args: &[&str],
        timeout: Duration,
    ) -> Result<(i32, String, String), GitError> {
        let mut spec = ProcessSpec::new("git", working_dir);
        spec.args = args.iter().map(|s| (*s).to_string()).collect();
        spec.total_timeout = Some(timeout);
        spec.env_whitelist = vec![
            "PATH".to_string(),
            "SYSTEMROOT".to_string(),
            "COMSPEC".to_string(),
            "TEMP".to_string(),
            "TMP".to_string(),
        ];

        let mut custom = HashMap::new();
        custom.insert("GIT_CONFIG_NOSYSTEM".to_string(), "1".to_string());
        custom.insert(
            "GIT_CONFIG_GLOBAL".to_string(),
            if cfg!(windows) {
                "NUL".to_string()
            } else {
                "/dev/null".to_string()
            },
        );
        custom.insert("GIT_AUTHOR_NAME".to_string(), "Dandori Agent".to_string());
        custom.insert(
            "GIT_AUTHOR_EMAIL".to_string(),
            "agent@dandori.local".to_string(),
        );
        custom.insert(
            "GIT_COMMITTER_NAME".to_string(),
            "Dandori Agent".to_string(),
        );
        custom.insert(
            "GIT_COMMITTER_EMAIL".to_string(),
            "agent@dandori.local".to_string(),
        );
        spec.custom_env = custom;

        let mut rx = spawn(spec)?;
        let mut stdout_lines = Vec::new();
        let mut stderr_lines = Vec::new();
        let mut exit_code = None;

        while let Some(event) = rx.recv().await {
            match event {
                ProcessEvent::StdoutLine(line) => stdout_lines.push(line),
                ProcessEvent::StderrLine(line) => stderr_lines.push(line),
                ProcessEvent::Exit(code) => exit_code = Some(code),
            }
        }

        let code = exit_code.unwrap_or(-1);
        Ok((code, stdout_lines.join("\n"), stderr_lines.join("\n")))
    }

    /// Khởi tạo một git repository mới.
    ///
    /// # Errors
    /// Trả về `GitError` nếu khởi tạo thất bại.
    pub async fn init_repo(&self, default_branch: &str) -> Result<(), GitError> {
        let (code, _, stderr) = self
            .run_git(
                &self.repo_path,
                &["init", "-b", default_branch],
                DEFAULT_GIT_TIMEOUT,
            )
            .await?;
        if code != 0 {
            return Err(GitError::CommandFailed { code, stderr });
        }
        Ok(())
    }

    /// Thiết lập cấu hình git cho repo.
    ///
    /// # Errors
    /// Trả về `GitError` nếu thiết lập thất bại.
    pub async fn config(&self, key: &str, value: &str) -> Result<(), GitError> {
        let (code, _, stderr) = self
            .run_git(
                &self.repo_path,
                &["config", key, value],
                DEFAULT_GIT_TIMEOUT,
            )
            .await?;
        if code != 0 {
            return Err(GitError::CommandFailed { code, stderr });
        }
        Ok(())
    }

    /// Thêm toàn bộ tệp và tạo commit.
    ///
    /// # Errors
    /// Trả về `GitError` nếu add hoặc commit thất bại.
    pub async fn commit_all(&self, message: &str) -> Result<(), GitError> {
        let (code1, _, stderr1) = self
            .run_git(&self.repo_path, &["add", "-A"], DEFAULT_GIT_TIMEOUT)
            .await?;
        if code1 != 0 {
            return Err(GitError::CommandFailed {
                code: code1,
                stderr: stderr1,
            });
        }
        let (code2, _, stderr2) = self
            .run_git(
                &self.repo_path,
                &["commit", "-m", message],
                DEFAULT_GIT_TIMEOUT,
            )
            .await?;
        if code2 != 0 {
            return Err(GitError::CommandFailed {
                code: code2,
                stderr: stderr2,
            });
        }
        Ok(())
    }

    /// Lấy hash SHA của một ref.
    ///
    /// # Errors
    /// Trả về `GitError::CommandFailed` nếu ref không tồn tại.
    pub async fn rev_parse(&self, rev: &str) -> Result<String, GitError> {
        validate_ref_name(rev)?;
        let (code, stdout, stderr) = self
            .run_git(&self.repo_path, &["rev-parse", rev], DEFAULT_GIT_TIMEOUT)
            .await?;
        if code != 0 {
            return Err(GitError::CommandFailed { code, stderr });
        }
        Ok(stdout.trim().to_string())
    }

    /// Kiểm tra repository đã có commit nào chưa.
    ///
    /// # Errors
    /// Trả về `GitError` nếu không thể kiểm tra.
    pub async fn has_commits(&self) -> Result<bool, GitError> {
        let (code, _, _) = self
            .run_git(
                &self.repo_path,
                &["rev-parse", "--verify", "HEAD"],
                DEFAULT_GIT_TIMEOUT,
            )
            .await?;
        Ok(code == 0)
    }

    /// Kiểm tra xem một nhánh cục bộ có tồn tại hay không.
    ///
    /// # Errors
    /// Trả về `GitError` nếu xảy ra lỗi tiến trình.
    pub async fn branch_exists(&self, branch_name: &str) -> Result<bool, GitError> {
        validate_ref_name(branch_name)?;
        let ref_target = format!("refs/heads/{branch_name}");
        let (code, _, _) = self
            .run_git(
                &self.repo_path,
                &["show-ref", "--verify", "--quiet", &ref_target],
                DEFAULT_GIT_TIMEOUT,
            )
            .await?;
        Ok(code == 0)
    }

    /// Thêm một worktree mới cùng nhánh mới dựa trên commit/nhánh cơ sở.
    ///
    /// # Errors
    /// Trả về `GitError::NoCommits` nếu repo chưa có commit, hoặc `GitError::CommandFailed` nếu thất bại.
    pub async fn worktree_add(
        &self,
        worktree_path: &Path,
        branch_name: &str,
        base_commit_or_branch: &str,
    ) -> Result<(), GitError> {
        validate_ref_name(branch_name)?;
        validate_ref_name(base_commit_or_branch)?;
        if !self.has_commits().await? {
            return Err(GitError::NoCommits);
        }

        if let Some(parent) = worktree_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let wt_str = worktree_path.to_string_lossy().to_string();
        let (code, _, stderr) = self
            .run_git(
                &self.repo_path,
                &[
                    "worktree",
                    "add",
                    "-b",
                    branch_name,
                    &wt_str,
                    base_commit_or_branch,
                ],
                DEFAULT_GIT_TIMEOUT,
            )
            .await?;

        if code != 0 {
            return Err(GitError::CommandFailed { code, stderr });
        }

        Ok(())
    }

    /// Xóa worktree và prune.
    ///
    /// # Errors
    /// Trả về `GitError::WorktreeDirty` nếu worktree có thay đổi chưa commit mà không có `force`.
    pub async fn worktree_remove(&self, worktree_path: &Path, force: bool) -> Result<(), GitError> {
        if !force && worktree_path.exists() {
            let dirty_files = self.status_porcelain(worktree_path).await?;
            if !dirty_files.is_empty() {
                return Err(GitError::WorktreeDirty);
            }
        }

        let wt_str = worktree_path.to_string_lossy().to_string();
        let args = if force {
            vec!["worktree", "remove", "--force", &wt_str]
        } else {
            vec!["worktree", "remove", &wt_str]
        };

        let (code, _, stderr) = self
            .run_git(&self.repo_path, &args, DEFAULT_GIT_TIMEOUT)
            .await?;

        if code != 0 {
            // Nếu git worktree remove lỗi nhưng thư mục vẫn còn và người dùng muốn force dọn dẹp
            if force && worktree_path.exists() {
                let _ = tokio::fs::remove_dir_all(worktree_path).await;
            } else {
                return Err(GitError::CommandFailed { code, stderr });
            }
        }

        // Tự động prune worktree metadata
        let _ = self
            .run_git(&self.repo_path, &["worktree", "prune"], DEFAULT_GIT_TIMEOUT)
            .await;

        Ok(())
    }

    /// Lấy danh sách worktree hiện có của repository.
    ///
    /// # Errors
    /// Trả về `GitError` nếu lệnh thất bại.
    pub async fn worktree_list(&self) -> Result<Vec<WorktreeInfo>, GitError> {
        let (code, stdout, stderr) = self
            .run_git(
                &self.repo_path,
                &["worktree", "list", "--porcelain"],
                DEFAULT_GIT_TIMEOUT,
            )
            .await?;

        if code != 0 {
            return Err(GitError::CommandFailed { code, stderr });
        }

        let mut list = Vec::new();
        let mut cur_path: Option<PathBuf> = None;
        let mut cur_head: Option<String> = None;
        let mut cur_branch: Option<String> = None;

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                if let (Some(p), Some(h)) = (cur_path.take(), cur_head.take()) {
                    list.push(WorktreeInfo {
                        path: p,
                        head: h,
                        branch: cur_branch.take(),
                    });
                }
                continue;
            }

            if let Some(p) = line.strip_prefix("worktree ") {
                cur_path = Some(PathBuf::from(p));
            } else if let Some(h) = line.strip_prefix("HEAD ") {
                cur_head = Some(h.to_string());
            } else if let Some(b) = line.strip_prefix("branch refs/heads/") {
                cur_branch = Some(b.to_string());
            }
        }

        if let (Some(p), Some(h)) = (cur_path, cur_head) {
            list.push(WorktreeInfo {
                path: p,
                head: h,
                branch: cur_branch,
            });
        }

        Ok(list)
    }

    /// Lấy danh sách các tệp thay đổi hoặc chưa theo dõi (status --porcelain) trong thư mục.
    ///
    /// # Errors
    /// Trả về `GitError` nếu lệnh thất bại.
    pub async fn status_porcelain(&self, dir: &Path) -> Result<Vec<String>, GitError> {
        let (code, stdout, stderr) = self
            .run_git(dir, &["status", "--porcelain", "-u"], DEFAULT_GIT_TIMEOUT)
            .await?;

        if code != 0 {
            return Err(GitError::CommandFailed { code, stderr });
        }

        let lines = stdout
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect();

        Ok(lines)
    }

    /// Lấy bản diff giữa worktree và base_ref.
    ///
    /// # Errors
    /// Trả về `GitError` nếu lệnh thất bại.
    pub async fn diff(&self, worktree_path: &Path, base_ref: &str) -> Result<String, GitError> {
        validate_ref_name(base_ref)?;
        let (code, stdout, stderr) = self
            .run_git(
                worktree_path,
                &["diff", base_ref, "--"],
                DEFAULT_GIT_TIMEOUT,
            )
            .await?;

        if code != 0 {
            return Err(GitError::CommandFailed { code, stderr });
        }

        Ok(stdout)
    }

    /// Lấy bản ánh xạ trạng thái tệp (name-status) từ git diff.
    ///
    /// # Errors
    /// Trả về `GitError` nếu lệnh thất bại hoặc `base_ref` không hợp lệ.
    pub async fn diff_name_status(
        &self,
        worktree_path: &Path,
        base_ref: &str,
    ) -> Result<HashMap<String, (String, Option<String>)>, GitError> {
        validate_ref_name(base_ref)?;
        let (code, stdout, stderr) = self
            .run_git(
                worktree_path,
                &["diff", "-M", "--name-status", base_ref, "--"],
                DEFAULT_GIT_TIMEOUT,
            )
            .await?;

        if code != 0 {
            return Err(GitError::CommandFailed { code, stderr });
        }

        let mut map = HashMap::new();
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split('\t').collect();
            let Some(status_code) = parts.first() else {
                continue;
            };
            let status = if status_code.starts_with('A') {
                "added".to_string()
            } else if status_code.starts_with('D') {
                "deleted".to_string()
            } else if status_code.starts_with('R') {
                "renamed".to_string()
            } else {
                "modified".to_string()
            };

            if status_code.starts_with('R') && parts.len() >= 3 {
                if let (Some(old_p), Some(new_p)) = (parts.get(1), parts.get(2)) {
                    map.insert((*new_p).to_string(), (status, Some((*old_p).to_string())));
                }
            } else if let Some(path) = parts.get(1) {
                map.insert((*path).to_string(), (status, None));
            }
        }
        Ok(map)
    }

    /// Lấy thống kê số dòng thay đổi theo từng tệp (numstat) giữa worktree và `base_ref`.
    ///
    /// # Errors
    /// Trả về `GitError` nếu lệnh thất bại hoặc `base_ref` không hợp lệ.
    pub async fn diff_numstat(
        &self,
        worktree_path: &Path,
        base_ref: &str,
    ) -> Result<Vec<crate::diff::DiffFileStat>, GitError> {
        validate_ref_name(base_ref)?;
        let (code, stdout, stderr) = self
            .run_git(
                worktree_path,
                &["diff", "-M", "--numstat", base_ref, "--"],
                DEFAULT_GIT_TIMEOUT,
            )
            .await?;

        if code != 0 {
            return Err(GitError::CommandFailed { code, stderr });
        }

        let name_status_map = self.diff_name_status(worktree_path, base_ref).await?;
        let mut stats = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some((path, old_from_numstat, ins, del, is_binary)) =
                crate::diff::parse_numstat_line(line)
            {
                let (status, old_path) = match name_status_map.get(&path) {
                    Some((st, old)) => (st.clone(), old.clone().or(old_from_numstat)),
                    None => {
                        let st = if old_from_numstat.is_some() {
                            "renamed".to_string()
                        } else {
                            "modified".to_string()
                        };
                        (st, old_from_numstat)
                    }
                };

                stats.push(crate::diff::DiffFileStat {
                    path,
                    old_path,
                    status,
                    insertions: ins,
                    deletions: del,
                    is_binary,
                });
            }
        }

        Ok(stats)
    }

    /// Lấy nội dung diff chi tiết của một tệp cụ thể.
    ///
    /// # Errors
    /// Trả về `GitError` nếu đường dẫn không an toàn hoặc lệnh git thất bại.
    pub async fn diff_file(
        &self,
        worktree_path: &Path,
        base_ref: &str,
        file_path: &str,
    ) -> Result<String, GitError> {
        validate_ref_name(base_ref)?;
        crate::diff::validate_path_inside_worktree(worktree_path, file_path)?;
        let (code, stdout, stderr) = self
            .run_git(
                worktree_path,
                &["diff", "-M", base_ref, "--", file_path],
                DEFAULT_GIT_TIMEOUT,
            )
            .await?;

        if code != 0 {
            return Err(GitError::CommandFailed { code, stderr });
        }

        Ok(stdout)
    }

    /// Lấy kết quả diff hoàn chỉnh và có cấu trúc giữa worktree và `base_ref`.
    ///
    /// # Errors
    /// Trả về `GitError` nếu lệnh git thất bại hoặc `base_ref` không hợp lệ.
    pub async fn diff_structured(
        &self,
        worktree_path: &Path,
        base_ref: &str,
        max_file_bytes: usize,
    ) -> Result<crate::diff::DiffResult, GitError> {
        let stats = self.diff_numstat(worktree_path, base_ref).await?;
        let mut files = Vec::new();
        let mut total_insertions = 0;
        let mut total_deletions = 0;

        for stat in &stats {
            total_insertions += stat.insertions;
            total_deletions += stat.deletions;

            let (diff_content, truncated) = if stat.is_binary {
                ("[Tệp nhị phân]".to_string(), false)
            } else {
                let raw_diff = self.diff_file(worktree_path, base_ref, &stat.path).await?;
                if raw_diff.len() > max_file_bytes {
                    let mut cut = raw_diff.chars().take(max_file_bytes).collect::<String>();
                    cut.push_str("\n[Nội dung diff đã được cắt bớt do vượt quá kích thước tối đa]");
                    (cut, true)
                } else {
                    (raw_diff, false)
                }
            };

            files.push(crate::diff::DiffFile {
                path: stat.path.clone(),
                old_path: stat.old_path.clone(),
                status: stat.status.clone(),
                insertions: stat.insertions,
                deletions: stat.deletions,
                is_binary: stat.is_binary,
                diff: diff_content,
                truncated,
            });
        }

        Ok(crate::diff::DiffResult {
            base_ref: base_ref.to_string(),
            files,
            stats: crate::diff::DiffStats {
                total_insertions,
                total_deletions,
                total_files_changed: stats.len(),
            },
        })
    }
}
