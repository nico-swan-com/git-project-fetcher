use crate::config::ProjectConfig;
use crate::error::{ProjectError};
use crate::git_utils::{
    checkout_branch, clone_repo, get_current_branch, is_git_repo, pull_branch_updates,
};
use crate::logger::{log_to_file, LogLevel};
use indicatif::ProgressBar;
use shellexpand;
use std::fs;
use std::path::{Path, PathBuf};

pub fn process_project(
    config: &ProjectConfig,
    parent_clone_dir: &Path,
    progress_bar: &ProgressBar,
) -> Result<(), ProjectError> {
    let expanded_project_path_str = shellexpand::tilde(&config.path).to_string();
    let project_path = if Path::new(&expanded_project_path_str).is_absolute() {
        PathBuf::from(expanded_project_path_str)
    } else {
        parent_clone_dir.join(&expanded_project_path_str) // Added &
    };

    let initial_msg = format!(
        "Checking project: '{}' at '{}'",
        config.project,
        project_path.display()
    );
    log_to_file(LogLevel::Info, &initial_msg);

    if !project_path.exists() {
        let msg = format!(
            "Project directory '{}' for '{}' not found. Attempting to clone.",
            project_path.display(),
            config.project
        );
        progress_bar.set_message(msg.clone());
        log_to_file(LogLevel::Info, &msg);

        if let Some(parent) = project_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| ProjectError::CreateDirs {
                    project_name: config.project.clone(),
                    path: parent.to_path_buf(),
                    source: e,
                })?;
            }
        }

        clone_repo(
            progress_bar,
            &config.project,
            &config.url,
            &project_path,
        )
        .map_err(|e| {
            log_to_file(
                LogLevel::Error,
                &format!(
                    "Clone failed for '{}'. Skipping project. Error: {}",
                    config.project, e
                ),
            );
            ProjectError::GitOperation {
                project_name: config.project.clone(),
                source: e,
            }
        })?;
    } else {
        let msg = format!(
            "Project directory '{}' for '{}' already exists.",
            project_path.display(),
            config.project
        );
        progress_bar.set_message(msg.clone());
        log_to_file(LogLevel::Info, &msg);
    }

    let check_repo_msg = format!(
        "Verifying Git repository at '{}'...",
        project_path.display()
    );
    progress_bar.set_message(check_repo_msg.clone());
    log_to_file(LogLevel::Info, &check_repo_msg);

    if !is_git_repo(&project_path) {
        let err_msg = format!(
            "'{}' is not a git repository. Skipping further git operations.",
            project_path.display()
        );
        progress_bar.set_message(err_msg.clone());
        log_to_file(LogLevel::Warning, &err_msg);
        // This specific case is more of a warning and skip, rather than a hard error for the whole app.
        // Depending on strictness, one might return an error here.
        // For now, log and return Ok to allow other projects to process.
        return Err(ProjectError::NotGitRepository {
             project_name: config.project.clone(),
             path: project_path.to_path_buf()
        });
    }

    // --- Git Pull Section ---
    if let Some(branches_to_pull) = &config.pull_branches {
        if !branches_to_pull.is_empty() {
            let original_branch = match get_current_branch(&project_path, &config.project) {
                Ok(branch) => {
                    log_to_file(
                        LogLevel::Info,
                        &format!(
                            "Project '{}': Current branch is '{}'.",
                            config.project, branch
                        ),
                    );
                    Some(branch)
                }
                Err(e) => {
                    let err_msg = format!(
                        "Project '{}': Could not determine current branch. Error: {}. Will proceed without restoring branch.",
                        config.project, e
                    );
                    progress_bar.set_message(format!("{} - Branch check failed", config.project));
                    log_to_file(LogLevel::Warning, &err_msg);
                    None
                }
            };

            for branch_name in branches_to_pull {
                progress_bar.set_message(format!(
                    "{} - Switching to branch {}",
                    config.project, branch_name
                ));
                match checkout_branch(
                    &project_path,
                    branch_name,
                    &config.project,
                    progress_bar,
                ) {
                    Ok(_) => {
                        if let Err(e) = pull_branch_updates(
                            &project_path,
                            Some(branch_name),
                            &config.project,
                            progress_bar,
                        ) {
                            log_to_file(
                                LogLevel::Warning,
                                &format!(
                                    "Project '{}': Continuing after pull error on branch {}: {}",
                                    config.project, branch_name, e
                                ),
                            );
                        }
                    }
                    Err(e) => {
                        let err_msg = format!(
                            "Project '{}': Failed to checkout branch '{}'. Skipping pull for this branch. Error: {}",
                            config.project, branch_name, e
                        );
                        progress_bar.set_message(format!(
                            "{} - Checkout failed: {}",
                            config.project, branch_name
                        ));
                        log_to_file(LogLevel::Error, &err_msg);
                    }
                }
            }

            if let Some(orig_branch_name) = original_branch {
                // Check if current branch is different from original, or if original wasn't in pull_branches list
                let current_branch_after_pulls = get_current_branch(&project_path, &config.project).ok();
                if current_branch_after_pulls.as_deref() != Some(&orig_branch_name) {
                    log_to_file(LogLevel::Info, &format!("Project '{}': Attempting to restore original branch '{}'.", config.project, orig_branch_name));
                    progress_bar.set_message(format!(
                        "{} - Restoring original branch {}",
                        config.project, orig_branch_name
                    ));
                    if let Err(e) = checkout_branch(
                        &project_path,
                        &orig_branch_name,
                        &config.project,
                        progress_bar,
                    ) {
                        let err_msg = format!(
                            "Project '{}': Failed to restore original branch '{}'. Error: {}",
                            config.project, orig_branch_name, e
                        );
                        progress_bar.set_message(format!(
                            "{} - Restore failed: {}",
                            config.project, orig_branch_name
                        ));
                        log_to_file(LogLevel::Warning, &err_msg);
                    }
                } else {
                     log_to_file(LogLevel::Info, &format!("Project '{}': Already on original branch '{}' or no restoration needed.", config.project, orig_branch_name));
                }
            }
        } else {
            let current_branch_for_log =
                get_current_branch(&project_path, &config.project).unwrap_or_else(|_| "current".to_string());
            log_to_file(
                LogLevel::Info,
                &format!(
                    "Project '{}': pull_branches is empty, pulling {} branch.",
                    config.project, current_branch_for_log
                ),
            );
            if let Err(e) =
                pull_branch_updates(&project_path, None, &config.project, progress_bar)
            {
                log_to_file(
                    LogLevel::Warning,
                    &format!("Project '{}': Continuing after pull error on current branch: {}", config.project, e),
                );
            }
        }
    } else {
        let current_branch_for_log =
            get_current_branch(&project_path, &config.project).unwrap_or_else(|_| "current".to_string());
        log_to_file(
            LogLevel::Info,
            &format!(
                "Project '{}': no pull_branches specified, pulling {} branch.",
                config.project, current_branch_for_log
            ),
        );
        if let Err(e) = pull_branch_updates(&project_path, None, &config.project, progress_bar) {
            log_to_file(
                LogLevel::Warning,
                &format!("Project '{}': Continuing after pull error on current branch: {}",config.project,  e),
            );
        }
    }

    let success_msg = format!("Finished checking/updating project: {}", config.project);
    log_to_file(LogLevel::Success, &success_msg);
    Ok(())
}