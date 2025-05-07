use crate::error::ConfigError;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub global_config: Option<GlobalConfig>,
    pub projects: Vec<ProjectConfig>,
}

#[derive(Deserialize, Debug, Default)]
pub struct GlobalConfig {
    pub default_clone_parent_directory: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ProjectConfig {
    pub project: String,
    pub url: String,
    pub path: String,
    pub pull_branches: Option<Vec<String>>,
}

pub fn load_config_from_file(config_file_path: &Path) -> Result<AppConfig, ConfigError> {
    if !config_file_path.exists() {
        return Err(ConfigError::NotFound(config_file_path.to_path_buf()));
    }

    let config_content = fs::read_to_string(config_file_path)
        .map_err(|e| ConfigError::ReadFile(config_file_path.to_path_buf(), e))?;

    let app_config: AppConfig =
        serde_json::from_str(&config_content).map_err(ConfigError::Parse)?;

    if app_config.projects.is_empty() {
        return Err(ConfigError::NoProjects);
    }

    for project_config in &app_config.projects {
        validate_project_config(project_config)?;
    }

    Ok(app_config)
}

pub fn validate_project_config(config: &ProjectConfig) -> Result<(), ConfigError> {
    if config.project.is_empty() {
        return Err(ConfigError::Validation {
            project_name: "Unknown (empty name)".to_string(),
            message: "Project name cannot be empty".to_string(),
        });
    }
    if config.url.is_empty() {
        return Err(ConfigError::Validation {
            project_name: config.project.clone(),
            message: "Repository URL cannot be empty".to_string(),
        });
    }
    if config.path.is_empty() {
        return Err(ConfigError::Validation {
            project_name: config.project.clone(),
            message: "Path cannot be empty".to_string(),
        });
    }
    Ok(())
}