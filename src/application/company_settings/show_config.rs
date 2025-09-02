#![allow(dead_code)]

use crate::domain::company_settings::config::Config;

#[allow(dead_code)]
pub struct ShowCompanyConfigUseCase;

impl ShowCompanyConfigUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, config: &Config) -> String {
        config.summary()
    }
}
