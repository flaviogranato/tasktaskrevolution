use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigManifest {
    #[serde(rename = "apiVersion")]
    pub api_version: String,
    pub kind: String,
    pub metadata: ConfigMetadata,
    pub spec: ConfigSpec,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigMetadata {
    pub name: String,
    #[serde(rename = "managerName")]
    pub manager_name: String,
    #[serde(rename = "managerEmail")]
    pub manager_email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigSpec {
    pub currency: String,
    pub work_hours_per_day: u8,
    pub work_days_per_week: Vec<String>,
    pub date_format: String,
    pub default_task_duration: u8,
    pub locale: String,
}
