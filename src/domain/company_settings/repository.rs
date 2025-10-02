#![allow(dead_code)]

use crate::domain::company_settings::config::Config;
use crate::domain::shared::errors::{DomainError, DomainResult};
use std::path::{Path, PathBuf};

pub trait ConfigRepository {
    fn save(&self, config: Config, path: &Path) -> DomainResult<()>;
    fn create_repository_dir(&self, path: &Path) -> DomainResult<()>;
    fn load(&self) -> DomainResult<(Config, PathBuf)>;
}
