use chrono::NaiveDate;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Task {
    pub id: String,
    pub project_code: String,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub start_date: NaiveDate,
    pub due_date: NaiveDate,
    pub actual_end_date: Option<NaiveDate>,
    pub assigned_resources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum TaskStatus {
    Completed,
    Planned,
    InProgress { progress: u8 },
    Blocked { reason: String },
    Cancelled,
}

#[allow(dead_code)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

#[allow(dead_code)]
impl DateRange {
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start <= other.end && self.end >= other.start
    }
}

#[allow(dead_code)]
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
                write!(f, "Recurso {res} está de férias neste período.")
            }
            TaskError::MissingField(field) => {
                write!(f, "Campo obrigatório não informado: {field}")
            }
        }
    }
}

impl std::error::Error for TaskError {}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    // Helper to create a date for tests
    fn d(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    #[test]
    fn test_date_range_overlaps() {
        let base_range = DateRange {
            start: d(2024, 1, 10),
            end: d(2024, 1, 20),
        };

        // --- Overlapping Cases ---
        // Other range is completely inside base_range
        let inside_range = DateRange {
            start: d(2024, 1, 12),
            end: d(2024, 1, 18),
        };
        assert!(base_range.overlaps(&inside_range));

        // Other range overlaps at the end
        let end_overlap = DateRange {
            start: d(2024, 1, 15),
            end: d(2024, 1, 25),
        };
        assert!(base_range.overlaps(&end_overlap));

        // Other range overlaps at the start
        let start_overlap = DateRange {
            start: d(2024, 1, 5),
            end: d(2024, 1, 15),
        };
        assert!(base_range.overlaps(&start_overlap));

        // Other range completely contains base_range
        let contains_range = DateRange {
            start: d(2024, 1, 5),
            end: d(2024, 1, 25),
        };
        assert!(base_range.overlaps(&contains_range));

        // Ranges are touching at the end date
        let touching_end = DateRange {
            start: d(2024, 1, 20),
            end: d(2024, 1, 25),
        };
        assert!(base_range.overlaps(&touching_end));

        // Ranges are touching at the start date
        let touching_start = DateRange {
            start: d(2024, 1, 5),
            end: d(2024, 1, 10),
        };
        assert!(base_range.overlaps(&touching_start));

        // --- Non-Overlapping Cases ---
        // Other range is completely after base_range
        let after_range = DateRange {
            start: d(2024, 1, 21),
            end: d(2024, 1, 25),
        };
        assert!(!base_range.overlaps(&after_range));

        // Other range is completely before base_range
        let before_range = DateRange {
            start: d(2024, 1, 1),
            end: d(2024, 1, 9),
        };
        assert!(!base_range.overlaps(&before_range));
    }

    #[test]
    fn test_task_error_display_formatting() {
        let invalid_date_err = TaskError::InvalidDateRange;
        assert_eq!(format!("{invalid_date_err}"), "Data inicial é posterior à data final.");

        let vacation_err = TaskError::ResourceOnVacation("RES-123".to_string());
        assert_eq!(
            format!("{vacation_err}"),
            "Recurso RES-123 está de férias neste período."
        );

        let missing_field_err = TaskError::MissingField("name");
        assert_eq!(format!("{missing_field_err}"), "Campo obrigatório não informado: name");
    }
}
