use crate::domain::shared_kernel::convertable::Convertable;
use crate::domain::task::Task;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: TaskMetadata,
    pub spec: TaskSpec,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskMetadata {
    #[serde(with = "datetime_format")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "date_format")]
    pub due_date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSpec {
    pub title: String,
    pub description: String,
    pub completed: bool,
}

impl Convertable<Task> for TaskManifest {
    fn from(source: Task) -> Self {
        TaskManifest {
            api_version: "tasktaskrevolution.io/v1alpha1".to_string(),
            kind: "Task".to_string(),
            metadata: TaskMetadata {
                created_at: Utc::now(),
                due_date: source.due_date(),
            },
            spec: TaskSpec {
                title: source.title().to_string(),
                description: source.description().to_string(),
                completed: source.is_completed(),
            },
        }
    }

    fn to(&self) -> Task {
        Task::new(
            self.spec.title.clone(),
            self.spec.description.clone(),
            self.metadata.due_date,
        )
        .unwrap() // TODO: Handle this error properly
    }
}

mod datetime_format {
    use chrono::{DateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format("%Y-%m-%dT%H:%M"));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_rfc3339(&format!("{}:00Z", s))
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(serde::de::Error::custom)
    }
}

mod date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format("%Y-%m-%d"));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_task_manifest_serialization() {
        let task = Task::new(
            "Test Task".to_string(),
            "Test Description".to_string(),
            Utc::now().naive_utc().date(),
        )
        .unwrap();

        let manifest = <TaskManifest as Convertable<Task>>::from(task);

        let yaml = serde_yaml::to_string(&manifest).unwrap();
        println!("{}", yaml);

        let manifest: TaskManifest = serde_yaml::from_str(&yaml).unwrap();
        let task = manifest.to();

        assert_eq!(task.title(), "Test Task");
        assert_eq!(task.description(), "Test Description");
    }
}
