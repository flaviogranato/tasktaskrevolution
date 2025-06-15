use crate::domain::project_management::layoff_period::LayoffPeriod;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct VacationRules {
    pub max_concurrent_vacations: Option<u32>,
    pub allow_layoff_vacations: Option<bool>,
    pub require_layoff_vacation_period: Option<bool>,
    pub layoff_periods: Option<Vec<LayoffPeriod>>,
}

impl Display for VacationRules {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "VacationRules {{ max_concurrent_vacations: {:?}, allow_layoff_vacations: {:?}, require_layoff_vacation_period: {:?}, layoff_periods: {:?} }}",
            self.max_concurrent_vacations, self.allow_layoff_vacations, self.require_layoff_vacation_period, self.layoff_periods
        )
    }
}
