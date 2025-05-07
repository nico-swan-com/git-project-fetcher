use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file {0}: {1}")]
    ReadFile(PathBuf, #[source] std::io::Error),
    #[error("Failed to parse config file: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("Validation error for project '{project_name}': {message}")]
    Validation { project_name: String, message: String },
    #[error("Configuration file '{0}' not found.")]
    NotFound(PathBuf),
    #[error("Configuration file is empty or contains no projects.")]
    NoProjects,
}

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Failed to execute git command for '{project_name}'. Command: '{command}'. IO Error: {source}")]
    CommandExecution {
        project_name: String,
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Git command failed for '{project_name}'. Command: '{command}'.\nStderr: {stderr}\nStdout: {stdout}")]
    CommandFailed {
        project_name: String,
        command: String,
        stdout: String,
        stderr: String,
    },
    #[error("Failed to get current branch for '{project_name}': {message}")]
    BranchInfoError { project_name: String, message: String },
}

#[derive(Error, Debug)]
pub enum ProjectError {
    // Removed PathResolutionError variant
    #[error("Project '{project_name}': Failed to create parent directories for '{path}': {source}")]
    CreateDirs {
        project_name: String,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Project '{project_name}': Git operation failed: {source}")]
    GitOperation {
        project_name: String,
        #[source]
        source: GitError,
    },
    #[error("Project '{project_name}': Non-Git directory found at '{path}', or clone failed earlier.")]
    NotGitRepository { project_name: String, path: PathBuf },
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    #[error("Project processing error: {0}")]
    Project(#[from] ProjectError),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Ctrl-C handler setup failed: {0}")]
    CtrlCSetup(#[from] ctrlc::Error),
    #[error("Failed to get current working directory: {0}")]
    CurrentDir(#[source] std::io::Error),
}