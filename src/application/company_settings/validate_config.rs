#![allow(dead_code)]

use crate::domain::company_settings::config::Config;

#[allow(dead_code)]
pub struct ValidateCompanyConfigUseCase;

impl ValidateCompanyConfigUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, config: &Config) -> bool {
        config.is_valid()
    }
}
