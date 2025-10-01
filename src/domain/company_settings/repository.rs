#![allow(dead_code)]

use crate::domain::shared::errors::{DomainError, DomainResult};
use crate::domain::company_settings::config::Config;
use std::path::PathBuf;

pub trait ConfigRepository {
    fn save(&self, config: Config, path: PathBuf) -> DomainResult<()>;
    fn create_repository_dir(&self, path: PathBuf) -> DomainResult<()>;
    fn load(&self) -> DomainResult<(Config, PathBuf)>;
}
