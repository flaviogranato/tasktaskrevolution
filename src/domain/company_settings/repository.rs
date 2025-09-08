#![allow(dead_code)]

use crate::domain::company_settings::config::Config;
use crate::application::errors::AppError;
use crate::infrastructure::persistence::manifests::config_manifest::ConfigManifest;
use std::path::PathBuf;

pub trait ConfigRepository {
    fn save(&self, config: ConfigManifest, path: PathBuf) -> Result<(), AppError>;
    fn create_repository_dir(&self, path: PathBuf) -> Result<(), AppError>;
    fn load(&self) -> Result<(Config, PathBuf), AppError>;
}
