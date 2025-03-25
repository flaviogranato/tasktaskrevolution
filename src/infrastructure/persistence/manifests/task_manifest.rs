use crate::domain::shared_kernel::convertable::Convertable;
use crate::domain::task::Task;
use chrono::{DateTime, Utc};
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
    pub id: String,
    #[serde(with = "datetime_format")]
    pub created_at: DateTime<Utc>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "datetime_format::optional"
    )]
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskSpec {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Convertable<Task> for TaskManifest {
    fn from(source: Task) -> Self {
        TaskManifest {
            api_version: "tasktaskrevolution.io/v1alpha1".to_string(),
            kind: "Task".to_string(),
            metadata: TaskMetadata {
                id: source.id,
                created_at: source.created_at,
                due_date: source.due_date,
            },
            spec: TaskSpec {
                name: source.name,
                description: source.description,
            },
        }
    }

    fn to(self) -> Task {
        Task {
            id: self.metadata.id,
            name: self.spec.name,
            description: self.spec.description,
            created_at: self.metadata.created_at,
            due_date: self.metadata.due_date,
        }
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

    pub mod optional {
        use super::*;

        pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match date {
                Some(date) => {
                    let s = format!("{}", date.format("%Y-%m-%d"));
                    serializer.serialize_str(&s)
                }
                None => serializer.serialize_none(),
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            Option::deserialize(deserializer)?
                .map(|s: String| {
                    DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", s))
                        .map(|dt| dt.with_timezone(&Utc))
                        .map_err(serde::de::Error::custom)
                })
                .transpose()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn format_datetime(dt: DateTime<Utc>) -> String {
        format!("{}", dt.format("%Y-%m-%dT%H:%M"))
    }

    #[test]
    fn test_task_manifest_serialization() {
        let mut task = Task::new(
            "Test Task".to_string(),
            Some("Test Description".to_string()),
            Some(Utc.with_ymd_and_hms(2024, 3, 15, 0, 0, 0).unwrap()),
        );
        task.id = "task-1".to_string();
        let manifest = <TaskManifest as Convertable<Task>>::from(task);

        let yaml = serde_yaml::to_string(&manifest).unwrap();

        let expected_yaml = format!(
            r#"apiVersion: tasktaskrevolution.io/v1alpha1
kind: Task
metadata:
  id: task-1
  createdAt: {}
  dueDate: 2024-03-15
spec:
  name: Test Task
  description: Test Description"#,
            format_datetime(manifest.metadata.created_at)
        );

        assert_eq!(yaml.trim(), expected_yaml);
    }

    #[test]
    fn test_task_manifest_deserialization() {
        let yaml = r#"
            apiVersion: tasktaskrevolution.io/v1alpha1
            kind: Task
            metadata:
              id: task-1
              createdAt: "2024-01-01T00:00"
              dueDate: "2024-03-15"
            spec:
              name: Test Task
              description: Test Description
        "#;

        let manifest: TaskManifest = serde_yaml::from_str(yaml).unwrap();

        assert_eq!(manifest.api_version, "tasktaskrevolution.io/v1alpha1");
        assert_eq!(manifest.kind, "Task");
        assert_eq!(manifest.metadata.id, "task-1");
        assert_eq!(
            manifest.metadata.created_at,
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
        );
        assert_eq!(
            manifest.metadata.due_date,
            Some(Utc.with_ymd_and_hms(2024, 3, 15, 0, 0, 0).unwrap())
        );
        assert_eq!(manifest.spec.name, "Test Task");
        assert_eq!(
            manifest.spec.description,
            Some("Test Description".to_string())
        );
    }

    #[test]
    fn test_task_manifest_without_optional_fields() {
        let mut task = Task::new("Test Task".to_string(), None, None);
        task.id = "task-1".to_string();
        let manifest = <TaskManifest as Convertable<Task>>::from(task);

        let yaml = serde_yaml::to_string(&manifest).unwrap();

        let expected_yaml = format!(
            r#"apiVersion: tasktaskrevolution.io/v1alpha1
kind: Task
metadata:
  id: task-1
  createdAt: {}
spec:
  name: Test Task"#,
            format_datetime(manifest.metadata.created_at)
        );

        assert_eq!(yaml.trim(), expected_yaml);
    }
}
