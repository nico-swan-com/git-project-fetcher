use std::env;
use std::path::{Path, PathBuf};

use indicatif::{ProgressBar, ProgressStyle};
use shellexpand;

mod config;
mod error;
mod git_utils;
mod logger;
mod project_logic;

use config::{load_config_from_file, AppConfig};
use error::AppError;
use logger::{log_to_file, LogLevel};
use project_logic::process_project;

fn main() -> Result<(), AppError> {
    let pb_for_ctrlc_dummy = ProgressBar::hidden(); // Keep dummy for ctrlc
    let pb_clone_for_ctrlc = pb_for_ctrlc_dummy.clone();

    ctrlc::set_handler(move || {
        pb_clone_for_ctrlc.finish_and_clear();
        println!("\nInterrupt received. Exiting...");
        log_to_file(LogLevel::Warning, "Process interrupted by user (Ctrl+C).");
        std::process::exit(0); // Exit the process
    })?; // Use ? for error propagation

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.contains(&"--help".to_string()) {
        println!("Usage: git_project_updater <config_file.json>");
        println!("A tool to clone and update multiple Git repositories based on a JSON config.");
        println!("\nConfig file format example:");
        println!(
            r#"
{{
  "global_config": {{
    "default_clone_parent_directory": "~/projects/work"
  }},
  "projects": [
    {{
      "project": "MyCoolApp",
      "url": "https://github.com/user/mycoolapp.git",
      "path": "mycoolapp",
      "pull_branches": ["main", "develop"]
    }},
    {{
      "project": "AnotherProject",
      "url": "https://github.com/user/anotherproject.git",
      "path": "/absolute/path/to/anotherproject"
    }},
    {{
      "project": "LegacySystem",
      "url": "https://github.com/user/legacysystem.git",
      "path": "old_stuff/legacy",
      "pull_branches": []
    }}
  ]
}}
"#
        );
        std::process::exit(0);
    }

    let config_file_path = Path::new(&args[1]);
    let app_config: AppConfig = load_config_from_file(config_file_path)?;

    let config_file_dir = config_file_path
        .parent()
        .unwrap_or_else(|| Path::new("."));
    let app_cwd = env::current_dir().map_err(AppError::CurrentDir)?;

    let effective_parent_dir_for_cloning: PathBuf = app_config
        .global_config
        .as_ref()
        .and_then(|gc| gc.default_clone_parent_directory.as_ref())
        .map_or_else(
            || config_file_dir.to_path_buf(),
            |parent_dir_str| {
                if parent_dir_str.is_empty() {
                    app_cwd
                } else {
                    let expanded_parent_dir = shellexpand::tilde(parent_dir_str).to_string();
                    let parent_path = Path::new(&expanded_parent_dir);
                    if parent_path.is_absolute() {
                        parent_path.to_path_buf()
                    } else {
                        config_file_dir.join(parent_path)
                    }
                }
            },
        );
    
    log_to_file(LogLevel::Info, &format!("Effective parent directory for relative project paths: {}", effective_parent_dir_for_cloning.display()));


    let overall_progress_bar = ProgressBar::new(app_config.projects.len() as u64);
    overall_progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green.bright} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {wide_msg}")
            .expect("Failed to set progress bar template"),
    );

    log_to_file(LogLevel::Info, "Starting project processing run.");
    let mut encountered_project_error = false;

    for project_config in app_config.projects {
        // No need to call validate_project_config here, it's done in load_config_from_file
        let processing_msg = format!("Starting: {}", project_config.project);
        overall_progress_bar.set_message(processing_msg.clone());

        match process_project(
            &project_config,
            &effective_parent_dir_for_cloning,
            &overall_progress_bar,
        ) {
            Ok(_) => {
                let completed_msg = format!("Done: {}", project_config.project);
                overall_progress_bar.set_message(completed_msg);
            }
            Err(e) => {
                encountered_project_error = true;
                let error_message = format!(
                    "Error processing project {}: {}",
                    project_config.project, e
                );

                log_to_file(LogLevel::Error, &error_message);

                overall_progress_bar.set_message(format!("Error: {} (see log)", project_config.project));

            }
        }
        overall_progress_bar.inc(1);
    }

    if encountered_project_error {
        overall_progress_bar.finish_with_message("Some projects encountered errors. Check project_fetcher.log for details.");
        log_to_file(LogLevel::Warning, "Finished project processing run with some errors.");
    } else {
        overall_progress_bar.finish_with_message("All projects processed successfully. Check project_fetcher.log for details.");
        log_to_file(LogLevel::Info, "Finished project processing run successfully.");
    }

    Ok(())
}