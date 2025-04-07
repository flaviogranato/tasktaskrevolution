use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct LayoffPeriod {
    pub start_date: String,
    pub end_date: String,
}

impl Display for LayoffPeriod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "LayoffPeriod {{ start_date: {}, end_date: {} }}",
            self.start_date, self.end_date
        )
    }
}
