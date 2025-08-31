use crate::domain::company_settings::config::Config;

pub struct ShowCompanyConfigUseCase;

impl ShowCompanyConfigUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, config: &Config) -> String {
        config.summary()
    }
}
