use anyhow::Result;
use std::path::Path;
use std::process::Command;

use crate::model::FileDiffSummary;

pub struct GitManager;

impl GitManager {
    pub fn current_branch(project_path: &Path) -> Option<String> {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .current_dir(project_path)
            .output()
            .ok()?;

        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !branch.empty_or_whitespace() {
                return Some(branch);
            }
        }
        None
    }

    pub fn current_head_sha(project_path: &Path) -> Option<String> {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .current_dir(project_path)
            .output()
            .ok()?;

        if output.status.success() {
            let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !sha.is_empty() {
                return Some(sha);
            }
        }
        None
    }

    pub fn get_diff_summary(project_path: &Path, base_sha: Option<&str>) -> (Vec<FileDiffSummary>, String) {
        let mut cmd = Command::new("git");
        cmd.arg("diff");
        if let Some(sha) = base_sha {
            cmd.arg(sha);
        }
        cmd.current_dir(project_path);

        let output = match cmd.output() {
            Ok(o) => o,
            Err(_) => return (Vec::new(), String::new()),
        };

        let diff_content = String::from_utf8_lossy(&output.stdout).to_string();

        let mut stat_cmd = Command::new("git");
        stat_cmd.arg("diff").arg("--numstat");
        if let Some(sha) = base_sha {
            stat_cmd.arg(sha);
        }
        stat_cmd.current_dir(project_path);

        let mut files = Vec::new();
        if let Ok(stat_out) = stat_cmd.output() {
            let stat_str = String::from_utf8_lossy(&stat_out.stdout);
            for line in stat_str.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let additions = parts[0].parse::<usize>().unwrap_or(0);
                    let deletions = parts[1].parse::<usize>().unwrap_or(0);
                    let file_path = parts[2..].join(" ");
                    files.push(FileDiffSummary {
                        path: file_path,
                        additions,
                        deletions,
                    });
                }
            }
        }

        (files, diff_content)
    }

    pub fn get_file_diff(project_path: &Path, file_path: &str) -> String {
        let mut cmd = Command::new("git");
        cmd.arg("diff").arg("--").arg(file_path);
        cmd.current_dir(project_path);

        match cmd.output() {
            Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
            Err(_) => String::new(),
        }
    }

    pub fn revert_to_sha(project_path: &Path, sha: &str) -> Result<()> {
        let _ = Command::new("git")
            .arg("reset")
            .arg("--hard")
            .arg(sha)
            .current_dir(project_path)
            .output();

        Ok(())
    }

    pub fn is_git_repository(project_path: &Path) -> bool {
        project_path.join(".git").exists()
    }
}

trait EmptyOrWhitespace {
    fn empty_or_whitespace(&self) -> bool;
}

impl EmptyOrWhitespace for str {
    fn empty_or_whitespace(&self) -> bool {
        self.trim().is_empty()
    }
}
