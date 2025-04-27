use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub start_date: NaiveDate,
    pub due_date: NaiveDate,
    pub actual_end_date: Option<NaiveDate>,
    pub assigned_resources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Completed,
    Planned,
    InProgress { progress: u8 },
    Blocked { reason: String },
    Cancelled,
}

pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl DateRange {
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start <= other.end && self.end >= other.start
    }
}

#[derive(Debug)]
pub enum TaskError {
    InvalidDateRange,
    ResourceOnVacation(String),
    MissingField(&'static str),
}

impl std::fmt::Display for TaskError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskError::InvalidDateRange => write!(f, "Data inicial é posterior à data final."),
            TaskError::ResourceOnVacation(res) => {
                write!(f, "Recurso {} está de férias neste período.", res)
            }
            TaskError::MissingField(field) => {
                write!(f, "Campo obrigatório não informado: {}", field)
            }
        }
    }
}

impl std::error::Error for TaskError {}
