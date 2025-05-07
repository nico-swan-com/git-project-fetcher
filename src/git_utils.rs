use crate::error::GitError;
use crate::logger::{log_to_file, LogLevel};
use indicatif::ProgressBar;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn is_git_repo(path: &Path) -> bool {
    path.join(".git").is_dir()
}

pub fn clone_repo(
    progress_bar: &ProgressBar,
    project_name: &str,
    repo_url: &str,
    target_path: &Path,
) -> Result<(), GitError> {
    let msg = format!(
        "Cloning '{}' from '{}' into '{}'...",
        project_name,
        repo_url,
        target_path.display()
    );
    progress_bar.set_message(msg.clone());
    log_to_file(LogLevel::Info, &msg);

    let output = Command::new("git")
        .arg("clone")
        .arg(repo_url)
        .arg(target_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| GitError::CommandExecution {
            project_name: project_name.to_string(),
            command: format!("git clone {} {}", repo_url, target_path.display()),
            source: e,
        })?;

    if output.status.success() {
        let success_msg = format!("Successfully cloned '{}'.", project_name);
        progress_bar.set_message(success_msg.clone());
        log_to_file(LogLevel::Success, &success_msg);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Err(GitError::CommandFailed {
            project_name: project_name.to_string(),
            command: format!("git clone {} {}", repo_url, target_path.display()),
            stdout,
            stderr,
        })
    }
}

pub fn get_current_branch(repo_path: &Path, project_name: &str) -> Result<String, GitError> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()
        .map_err(|e| GitError::CommandExecution {
            project_name: project_name.to_string(),
            command: "git rev-parse --abbrev-ref HEAD".to_string(),
            source: e,
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(GitError::BranchInfoError {
            project_name: project_name.to_string(),
            message: format!(
                "Failed to get current branch. Git stderr: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        })
    }
}

pub fn checkout_branch(
    repo_path: &Path,
    branch: &str,
    project_name: &str,
    progress_bar: &ProgressBar,
) -> Result<(), GitError> {
    let msg = format!(
        "Project '{}': Attempting to checkout branch '{}'...",
        project_name, branch
    );
    progress_bar.set_message(msg.clone());
    log_to_file(LogLevel::Info, &msg);

    let output = Command::new("git")
        .current_dir(repo_path)
        .arg("checkout")
        .arg(branch)
        .output()
        .map_err(|e| GitError::CommandExecution {
            project_name: project_name.to_string(),
            command: format!("git checkout {}", branch),
            source: e,
        })?;

    if output.status.success() {
        let success_msg = format!(
            "Project '{}': Successfully checked out branch '{}'.",
            project_name, branch
        );
        progress_bar.set_message(success_msg.clone());
        log_to_file(LogLevel::Success, &success_msg);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Err(GitError::CommandFailed {
            project_name: project_name.to_string(),
            command: format!("git checkout {}", branch),
            stdout,
            stderr,
        })
    }
}

pub fn pull_branch_updates( // Renamed from pull_branch to avoid conflict with Option<&str> branch
    repo_path: &Path,
    branch_to_pull: Option<&str>,
    project_name: &str,
    progress_bar: &ProgressBar,
) -> Result<(), GitError> {
    let branch_display_name = branch_to_pull.unwrap_or("current branch");
    let pull_msg = format!(
        "Project '{}': Pulling updates for {}...",
        project_name, branch_display_name
    );
    progress_bar.set_message(pull_msg.clone());
    log_to_file(LogLevel::Info, &pull_msg);

    let mut git_pull_cmd = Command::new("git");
    git_pull_cmd.current_dir(repo_path).arg("pull");

    let command_string = if let Some(branch) = branch_to_pull {
        git_pull_cmd.arg("origin").arg(branch);
        format!("git pull origin {}", branch)
    } else {
        "git pull".to_string()
    };

    let pull_output = git_pull_cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| GitError::CommandExecution {
            project_name: project_name.to_string(),
            command: command_string.clone(),
            source: e,
        })?;

    if pull_output.status.success() {
        let stdout_str = String::from_utf8_lossy(&pull_output.stdout);
        if stdout_str.contains("Already up to date.") || stdout_str.contains("Bereits aktuell.") {
            let msg = format!(
                "Project '{}' ({}) is already up to date.",
                project_name, branch_display_name
            );
            progress_bar.set_message(msg.clone());
            log_to_file(LogLevel::Info, &msg);
        } else {
            let msg = format!(
                "Project '{}': Successfully pulled updates for {}.",
                project_name, branch_display_name
            );
            progress_bar.set_message(msg.clone());
            log_to_file(LogLevel::Success, &msg);
            if !stdout_str.trim().is_empty() {
                log_to_file(
                    LogLevel::Info,
                    &format!(
                        "Git pull output for '{}' ({}):\n{}",
                        project_name,
                        branch_display_name,
                        stdout_str.trim()
                    ),
                );
            }
        }
        Ok(())
    } else {
        let stderr_str = String::from_utf8_lossy(&pull_output.stderr).trim().to_string();
        let stdout_str = String::from_utf8_lossy(&pull_output.stdout).trim().to_string();
        Err(GitError::CommandFailed {
            project_name: project_name.to_string(),
            command: command_string,
            stdout: stdout_str,
            stderr: stderr_str,
        })
    }
}