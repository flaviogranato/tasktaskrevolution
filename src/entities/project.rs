use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectManifest {
    api_version: String,
    kind: String,
    metadata: ProjectMetadata,
    spec: ProjectSpec,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProjectMetadata {
    name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ProjectSpec {
    start_date: Option<String>,
    end_date: Option<String>,
    status: ProjectStatus,
}

#[derive(Serialize, Deserialize, Debug)]
enum ProjectStatus {
    Planned,
    InProgress,
    Completed,
    Cancelled,
}

impl ProjectManifest {
    pub fn new(name: String, start_date: Option<String>, end_date: Option<String>) -> Self {
        ProjectManifest {
            api_version: "tasktaskrevolution.io/v1alpha1".to_string(),
            kind: "Project".to_string(),
            metadata: ProjectMetadata {
                name,
            },
            spec: ProjectSpec {
                start_date,
                end_date,
                status: ProjectStatus::Planned,
            },
        }
    }
}
