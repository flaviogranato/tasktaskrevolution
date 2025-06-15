use std::fmt::Display;

#[derive(Debug)]
pub struct Config {
    pub manager_name: String,
    pub manager_email: String,
}

impl Config {
    pub fn new(manager_name: String, manager_email: String) -> Self {
        Self { manager_name, manager_email }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Config {{ name: {}, email: {} }}",
            self.manager_name, self.manager_email
        )
    }
}
