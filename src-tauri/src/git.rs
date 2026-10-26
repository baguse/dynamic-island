use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GitCommitInfo {
    pub hash: String,
    pub short_hash: String,
    pub message: String,
    pub author: String,
    pub relative_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitFileChange {
    pub path: String,
    pub status: String,
    pub is_staged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GitStatus {
    pub is_repo: bool,
    pub repo_name: Option<String>,
    pub repo_path: Option<String>,
    pub branch: Option<String>,
    pub head_commit: Option<GitCommitInfo>,
    pub staged_count: usize,
    pub unstaged_count: usize,
    pub untracked_count: usize,
    pub ahead: usize,
    pub behind: usize,
    pub changed_files: Vec<GitFileChange>,
    pub recent_commits: Vec<GitCommitInfo>,
}

impl Default for GitStatus {
    fn default() -> Self {
        Self {
            is_repo: false,
            repo_name: None,
            repo_path: None,
            branch: None,
            head_commit: None,
            staged_count: 0,
            unstaged_count: 0,
            untracked_count: 0,
            ahead: 0,
            behind: 0,
            changed_files: Vec::new(),
            recent_commits: Vec::new(),
        }
    }
}

pub fn get_git_status(target_dir: Option<&Path>) -> GitStatus {
    let base_dir = if let Some(d) = target_dir {
        d.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    };

    // 1. Check if inside a Git work tree
    let is_inside = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .current_dir(&base_dir)
        .output();

    let is_repo = match is_inside {
        Ok(out) => String::from_utf8_lossy(&out.stdout).trim() == "true",
        Err(_) => false,
    };

    if !is_repo {
        return GitStatus::default();
    }

    // 2. Top-level repository directory
    let toplevel = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .current_dir(&base_dir)
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
            } else {
                None
            }
        });

    let repo_path = toplevel.clone();
    let repo_name = toplevel.as_ref().and_then(|p| {
        Path::new(p)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    });

    let work_dir = toplevel.as_ref().map(PathBuf::from).unwrap_or(base_dir);

    // 3. Branch detection
    let branch_out = Command::new("git")
        .arg("branch")
        .arg("--show-current")
        .current_dir(&work_dir)
        .output()
        .ok();

    let mut branch = branch_out
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty());

    if branch.is_none() {
        // Fallback for fresh repos before initial commit
        if let Ok(sym_out) = Command::new("git")
            .arg("symbolic-ref")
            .arg("--short")
            .arg("HEAD")
            .current_dir(&work_dir)
            .output()
        {
            if sym_out.status.success() {
                let name = String::from_utf8_lossy(&sym_out.stdout).trim().to_string();
                if !name.is_empty() {
                    branch = Some(name);
                }
            }
        }
    }

    if branch.is_none() {
        // Detached HEAD state
        if let Ok(head_out) = Command::new("git")
            .arg("rev-parse")
            .arg("--short")
            .arg("HEAD")
            .current_dir(&work_dir)
            .output()
        {
            if head_out.status.success() {
                let short = String::from_utf8_lossy(&head_out.stdout).trim().to_string();
                if !short.is_empty() {
                    branch = Some(format!("HEAD ({})", short));
                }
            }
        }
    }

    // 4. Status porcelain (staged, unstaged, untracked)
    let mut staged_count = 0;
    let mut unstaged_count = 0;
    let mut untracked_count = 0;
    let mut changed_files = Vec::new();

    if let Ok(status_out) = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(&work_dir)
        .output()
    {
        if status_out.status.success() {
            let stdout = String::from_utf8_lossy(&status_out.stdout);
            for line in stdout.lines() {
                if line.len() < 3 {
                    continue;
                }
                let staged_char = line.chars().next().unwrap_or(' ');
                let unstaged_char = line.chars().nth(1).unwrap_or(' ');
                let file_path = line[3..].trim().to_string();

                if line.starts_with("??") {
                    untracked_count += 1;
                    changed_files.push(GitFileChange {
                        path: file_path,
                        status: "??".to_string(),
                        is_staged: false,
                    });
                } else {
                    if staged_char != ' ' && staged_char != '?' {
                        staged_count += 1;
                        changed_files.push(GitFileChange {
                            path: file_path.clone(),
                            status: staged_char.to_string(),
                            is_staged: true,
                        });
                    }
                    if unstaged_char != ' ' && unstaged_char != '?' {
                        unstaged_count += 1;
                        changed_files.push(GitFileChange {
                            path: file_path,
                            status: unstaged_char.to_string(),
                            is_staged: false,
                        });
                    }
                }
            }
        }
    }

    // 5. Ahead / Behind tracking against upstream remote
    let mut ahead = 0;
    let mut behind = 0;
    if let Ok(rev_out) = Command::new("git")
        .arg("rev-list")
        .arg("--count")
        .arg("--left-right")
        .arg("@{upstream}...HEAD")
        .current_dir(&work_dir)
        .output()
    {
        if rev_out.status.success() {
            let s = String::from_utf8_lossy(&rev_out.stdout);
            let parts: Vec<&str> = s.trim().split_whitespace().collect();
            if parts.len() == 2 {
                behind = parts[0].parse().unwrap_or(0);
                ahead = parts[1].parse().unwrap_or(0);
            }
        }
    }

    // 6. Recent commits
    let mut recent_commits = Vec::new();
    if let Ok(log_out) = Command::new("git")
        .arg("log")
        .arg("-n")
        .arg("5")
        .arg("--pretty=format:%H\t%h\t%s\t%an\t%cr")
        .current_dir(&work_dir)
        .output()
    {
        if log_out.status.success() {
            let stdout = String::from_utf8_lossy(&log_out.stdout);
            for line in stdout.lines() {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 5 {
                    recent_commits.push(GitCommitInfo {
                        hash: parts[0].to_string(),
                        short_hash: parts[1].to_string(),
                        message: parts[2].to_string(),
                        author: parts[3].to_string(),
                        relative_time: parts[4].to_string(),
                    });
                }
            }
        }
    }

    let head_commit = recent_commits.first().cloned();

    GitStatus {
        is_repo: true,
        repo_name,
        repo_path,
        branch,
        head_commit,
        staged_count,
        unstaged_count,
        untracked_count,
        ahead,
        behind,
        changed_files,
        recent_commits,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_status_default() {
        let status = GitStatus::default();
        assert!(!status.is_repo);
        assert_eq!(status.staged_count, 0);
        assert_eq!(status.unstaged_count, 0);
    }

    #[test]
    fn test_git_status_current_dir() {
        let status = get_git_status(None);
        assert!(status.is_repo);
        assert!(status.branch.is_some());
        assert_eq!(status.repo_name.as_deref(), Some("dynamic-island"));
    }
}
