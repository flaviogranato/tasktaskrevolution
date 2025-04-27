use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceAssignment {
    pub resource_id: String,
    pub allocation_percentage: u8,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}
