use crate::domain::project::layoff_period::LayoffPeriod;
use crate::domain::project::model::{Project, ProjectStatus};
use crate::domain::project::vacation_rules::VacationRules;
use crate::domain::shared_kernel::convertable::Convertable;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectManifest {
    pub api_version: String,
    pub kind: String,
    pub metadata: ProjectMetadata,
    pub spec: ProjectSpec,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSpec {
    pub name: String,
    pub description: Option<String>,
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    pub status: ProjectStatusManifest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vacation_rules: Option<VacationRulesManifest>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VacationRulesManifest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrent_vacations: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_layoff_vacations: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_layoff_vacation_period: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layoff_periods: Option<Vec<LayoffPeriodManifest>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoffPeriodManifest {
    pub start_date: String,
    pub end_date: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ProjectStatusManifest {
    Planned,
    InProgress,
    Completed,
    Cancelled,
}

impl Convertable<ProjectStatus> for ProjectStatusManifest {
    fn from(source: ProjectStatus) -> Self {
        match source {
            ProjectStatus::Planned => ProjectStatusManifest::Planned,
            ProjectStatus::InProgress => ProjectStatusManifest::InProgress,
            ProjectStatus::Completed => ProjectStatusManifest::Completed,
            ProjectStatus::Cancelled => ProjectStatusManifest::Cancelled,
        }
    }

    fn to(&self) -> ProjectStatus {
        match self {
            ProjectStatusManifest::Planned => ProjectStatus::Planned,
            ProjectStatusManifest::InProgress => ProjectStatus::InProgress,
            ProjectStatusManifest::Completed => ProjectStatus::Completed,
            ProjectStatusManifest::Cancelled => ProjectStatus::Cancelled,
        }
    }
}

impl ProjectManifest {
    pub fn new(
        code: Option<String>,
        name: String,
        description: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> Self {
        ProjectManifest {
            api_version: "tasktaskrevolution.io/v1alpha1".to_string(),
            kind: "Project".to_string(),
            metadata: ProjectMetadata {
                name: name.clone(),
                code,
                description: description.clone(),
            },
            spec: ProjectSpec {
                name,
                description,
                timezone: None,
                start_date,
                end_date,
                status: ProjectStatusManifest::Planned,
                vacation_rules: Some(VacationRulesManifest {
                    max_concurrent_vacations: None,
                    allow_layoff_vacations: None,
                    require_layoff_vacation_period: None,
                    layoff_periods: None,
                }),
            },
        }
    }
}

impl Convertable<Project> for ProjectManifest {
    fn from(source: Project) -> Self {
        ProjectManifest {
            api_version: "tasktaskrevolution.io/v1alpha1".to_string(),
            kind: "Project".to_string(),
            metadata: ProjectMetadata {
                code: source.id.clone(),
                name: source.name.clone(),
                description: source.description.clone(),
            },
            spec: ProjectSpec {
                name: source.name,
                description: source.description,
                timezone: None,
                start_date: source.start_date,
                end_date: source.end_date,
                status: <ProjectStatusManifest as Convertable<ProjectStatus>>::from(source.status),
                vacation_rules: source
                    .vacation_rules
                    .map(<VacationRulesManifest as Convertable<VacationRules>>::from),
            },
        }
    }

    fn to(&self) -> Project {
        Project {
            id: None, // TODO: Você precisará gerar um ID aqui, se necessário
            name: self.metadata.name.clone(),
            description: self.metadata.description.clone(),
            start_date: self.spec.start_date.clone(),
            end_date: self.spec.end_date.clone(),
            status: <ProjectStatusManifest as Convertable<ProjectStatus>>::to(&self.spec.status),
            vacation_rules: self.spec.vacation_rules.as_ref().map(|vr| vr.to()),
        }
    }
}

impl Convertable<VacationRules> for VacationRulesManifest {
    fn from(source: VacationRules) -> Self {
        VacationRulesManifest {
            max_concurrent_vacations: source.max_concurrent_vacations,
            allow_layoff_vacations: source.allow_layoff_vacations,
            require_layoff_vacation_period: source.require_layoff_vacation_period,
            layoff_periods: source.layoff_periods.map(|periods| {
                periods
                    .into_iter()
                    .map(<LayoffPeriodManifest as Convertable<LayoffPeriod>>::from)
                    .collect()
            }),
        }
    }

    fn to(&self) -> VacationRules {
        VacationRules {
            max_concurrent_vacations: self.max_concurrent_vacations,
            allow_layoff_vacations: self.allow_layoff_vacations,
            require_layoff_vacation_period: self.require_layoff_vacation_period,
            layoff_periods: self.layoff_periods.as_ref()
                .map(|periods| periods.iter().map(|period| period.to()).collect()),
        }
    }
}

impl Convertable<LayoffPeriod> for LayoffPeriodManifest {
    fn from(source: LayoffPeriod) -> Self {
        LayoffPeriodManifest {
            start_date: source.start_date,
            end_date: source.end_date,
        }
    }

    fn to(&self) -> LayoffPeriod {
        LayoffPeriod {
            start_date: self.start_date.clone(),
            end_date: self.end_date.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layoff_period_serialize() {
        let layoff_period = LayoffPeriodManifest {
            start_date: "2024-01-01".to_string(),
            end_date: "2024-01-31".to_string(),
        };
        let serialized = serde_yaml::to_string(&layoff_period).unwrap();
        assert_eq!(serialized, "startDate: 2024-01-01\nendDate: 2024-01-31\n");
    }

    #[test]
    fn test_layoff_period_deserialize() {
        let yaml = "startDate: '2024-01-01'\nendDate: '2024-01-31'\n";
        let deserialized: LayoffPeriodManifest = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(deserialized.start_date, "2024-01-01");
        assert_eq!(deserialized.end_date, "2024-01-31");
    }

    #[test]
    fn test_vacation_rules_serialize_with_layoff_periods() {
        let vacation_rules = VacationRulesManifest {
            max_concurrent_vacations: Some(2),
            allow_layoff_vacations: Some(true),
            require_layoff_vacation_period: Some(true),
            layoff_periods: Some(vec![
                LayoffPeriodManifest {
                    start_date: "2024-01-01".to_string(),
                    end_date: "2024-01-31".to_string(),
                },
                LayoffPeriodManifest {
                    start_date: "2024-07-01".to_string(),
                    end_date: "2024-07-31".to_string(),
                },
            ]),
        };

        let serialized = serde_yaml::to_string(&vacation_rules).unwrap();
        let expected_yaml = r#"
maxConcurrentVacations: 2
allowLayoffVacations: true
requireLayoffVacationPeriod: true
layoffPeriods:
- startDate: 2024-01-01
  endDate: 2024-01-31
- startDate: 2024-07-01
  endDate: 2024-07-31
"#;
        assert_eq!(serialized.trim(), expected_yaml.trim());
    }

    #[test]
    fn test_vacation_rules_deserialize_with_layoff_periods() {
        let yaml = r#"
maxConcurrentVacations: 2
allowLayoffVacations: true
requireLayoffVacationPeriod: true
layoffPeriods:
- startDate: "2024-01-01"
  endDate: "2024-01-31"
- startDate: "2024-07-01"
  endDate: "2024-07-31"
"#;
        let deserialized: VacationRulesManifest = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(deserialized.max_concurrent_vacations, Some(2));
        assert_eq!(deserialized.allow_layoff_vacations, Some(true));
        assert_eq!(deserialized.require_layoff_vacation_period, Some(true));
        assert_eq!(deserialized.layoff_periods.as_ref().unwrap().len(), 2);
        assert_eq!(
            deserialized.layoff_periods.as_ref().unwrap()[0].start_date,
            "2024-01-01"
        );
        assert_eq!(
            deserialized.layoff_periods.as_ref().unwrap()[1].end_date,
            "2024-07-31"
        );
    }

    #[test]
    fn test_project_manifest_deserialize_with_full_data() {
        let yaml_str = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  code: ABC123
  name: Meu Projeto
  description: Descrição do Projeto
spec:
  name: Meu Projeto
  description: Descrição do Projeto
  timezone: null
  startDate: "2024-01-10"
  endDate: "2024-12-20"
  status: InProgress
  vacationRules:
    maxConcurrentVacations: 3
    allowLayoffVacations: true
    requireLayoffVacationPeriod: false
    layoffPeriods:
      - startDate: "2024-05-15"
        endDate: "2024-06-15"
      - startDate: "2024-11-01"
        endDate: "2024-11-30"
"#;

        let manifest: ProjectManifest = serde_yaml::from_str(yaml_str).unwrap();

        assert_eq!(manifest.api_version, "tasktaskrevolution.io/v1alpha1");
        assert_eq!(manifest.kind, "Project");
        assert_eq!(manifest.metadata.code, Some("ABC123".to_string()));
        assert_eq!(manifest.metadata.name, "Meu Projeto".to_string());
        assert_eq!(
            manifest.metadata.description,
            Some("Descrição do Projeto".to_string())
        );
        assert_eq!(manifest.spec.start_date, Some("2024-01-10".to_string()));
        assert_eq!(manifest.spec.end_date, Some("2024-12-20".to_string()));
        assert_eq!(manifest.spec.status, ProjectStatusManifest::InProgress);
        assert!(manifest.spec.vacation_rules.is_some());
        let vr = manifest.spec.vacation_rules.unwrap();
        assert_eq!(vr.max_concurrent_vacations, Some(3));
        assert_eq!(vr.allow_layoff_vacations, Some(true));
        assert_eq!(vr.require_layoff_vacation_period, Some(false));
        assert_eq!(vr.layoff_periods.as_ref().unwrap().len(), 2);
        assert_eq!(
            vr.layoff_periods.as_ref().unwrap()[0].start_date,
            "2024-05-15"
        );
        assert_eq!(
            vr.layoff_periods.as_ref().unwrap()[1].end_date,
            "2024-11-30"
        );
    }

    #[test]
    fn creates_manifest_with_all_fields() {
        let expected_code = Some("my-project-code".to_string());
        let expected_name = "My Project Name".to_string();
        let expected_description = Some("A cool project".to_string());
        let expected_start_date = Some("2025-01-01".to_string());
        let expected_end_date = Some("2025-12-31".to_string());

        let manifest = ProjectManifest::new(
            expected_code.clone(),
            expected_name.clone(),
            expected_description.clone(),
            expected_start_date.clone(),
            expected_end_date.clone(),
        );

        assert_eq!(
            manifest.api_version,
            "tasktaskrevolution.io/v1alpha1".to_string()
        );
        assert_eq!(manifest.kind, "Project".to_string());

        assert_eq!(manifest.metadata.name, expected_name);
        assert_eq!(manifest.metadata.code, expected_code);
        assert_eq!(manifest.metadata.description, expected_description);

        assert_eq!(manifest.spec.start_date, expected_start_date);
        assert_eq!(manifest.spec.end_date, expected_end_date);
        // assert_eq!(manifest.spec.status, ProjectStatus::Planned);
        assert_eq!(
            manifest
                .spec
                .vacation_rules
                .as_ref()
                .unwrap()
                .max_concurrent_vacations,
            None
        );
        assert_eq!(
            manifest
                .spec
                .vacation_rules
                .as_ref()
                .unwrap()
                .allow_layoff_vacations,
            None
        );
        assert_eq!(
            manifest
                .spec
                .vacation_rules
                .as_ref()
                .unwrap()
                .require_layoff_vacation_period,
            None
        );
        assert_eq!(
            manifest
                .spec
                .vacation_rules
                .as_ref()
                .unwrap()
                .layoff_periods,
            None
        );
    }

    #[test]
    fn creates_manifest_with_some_optional_fields() {
        let expected_name = "My Project Name".to_string();

        let manifest = ProjectManifest::new(None, expected_name.clone(), None, None, None);

        assert_eq!(
            manifest.api_version,
            "tasktaskrevolution.io/v1alpha1".to_string()
        );
        assert_eq!(manifest.kind, "Project".to_string());

        assert_eq!(manifest.metadata.name, expected_name);
        assert_eq!(manifest.metadata.code, None);
        assert_eq!(manifest.metadata.description, None);

        assert_eq!(manifest.spec.start_date, None);
        assert_eq!(manifest.spec.end_date, None);
        assert_eq!(manifest.spec.status, ProjectStatusManifest::Planned);
        assert_eq!(
            manifest
                .spec
                .vacation_rules
                .as_ref()
                .unwrap()
                .max_concurrent_vacations,
            None
        );
        assert_eq!(
            manifest
                .spec
                .vacation_rules
                .as_ref()
                .unwrap()
                .allow_layoff_vacations,
            None
        );
        assert_eq!(
            manifest
                .spec
                .vacation_rules
                .as_ref()
                .unwrap()
                .require_layoff_vacation_period,
            None
        );
        assert_eq!(
            manifest
                .spec
                .vacation_rules
                .as_ref()
                .unwrap()
                .layoff_periods,
            None
        );
    }

    #[test]
    fn creates_manifest_with_minimal_fields() {
        let expected_name = "Minimal Project".to_string();
        let manifest = ProjectManifest::new(None, expected_name.clone(), None, None, None);

        assert_eq!(manifest.metadata.name, expected_name);
        assert_eq!(manifest.metadata.code, None);
        assert_eq!(manifest.metadata.description, None);
        assert_eq!(manifest.spec.start_date, None);
        assert_eq!(manifest.spec.end_date, None);
        assert_eq!(manifest.spec.status, ProjectStatusManifest::Planned);
        assert!(manifest.spec.vacation_rules.is_some());
        let vr = manifest.spec.vacation_rules.unwrap();
        assert_eq!(vr.max_concurrent_vacations, None);
        assert_eq!(vr.allow_layoff_vacations, None);
        assert_eq!(vr.require_layoff_vacation_period, None);
        assert_eq!(vr.layoff_periods, None);
    }

    #[test]
    fn test_project_manifest_serialize_with_all_fields() {
        let expected_code = Some("my-project-code".to_string());
        let expected_name = "My Project Name".to_string();
        let expected_description = Some("A cool project".to_string());
        let expected_start_date = Some("2025-01-01".to_string());
        let expected_end_date = Some("2025-12-31".to_string());

        let manifest = ProjectManifest::new(
            expected_code.clone(),
            expected_name.clone(),
            expected_description.clone(),
            expected_start_date.clone(),
            expected_end_date.clone(),
        );

        let serialized_manifest = serde_yaml::to_string(&manifest).unwrap();

        let empty_string = String::new();
        let expected_yaml = format!(
            r#"apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  code: {}
  name: {}
  description: {}
spec:
  name: {}
  description: {}
  timezone: null
  startDate: {}
  endDate: {}
  status: Planned
  vacationRules: {{}}"#,
            expected_code.unwrap_or_else(|| "".to_string()),
            expected_name,
            expected_description.as_ref().unwrap_or(&empty_string),
            expected_name,
            expected_description.as_ref().unwrap_or(&empty_string),
            expected_start_date.unwrap_or_else(|| "".to_string()),
            expected_end_date.unwrap_or_else(|| "".to_string())
        );

        assert_eq!(serialized_manifest.trim(), expected_yaml.trim());
    }

    #[test]
    fn test_project_metadata_serialize_with_all_fields() {
        let code = Some("my-project-code".to_string());
        let name = "My Project Name".to_string();
        let description = Some("A cool project".to_string());

        let metadata = ProjectMetadata {
            code: code.clone(),
            name: name.clone(),
            description: description.clone(),
        };

        let serialized_metadata = serde_yaml::to_string(&metadata).unwrap();

        let expected_yaml = format!(
            r#"code: {}
name: {}
description: {}
"#,
            code.unwrap_or_else(|| "".to_string()),
            name,
            description.unwrap_or_else(|| "".to_string())
        );

        assert_eq!(serialized_metadata, expected_yaml);
    }

    #[test]
    fn test_project_spec_serialize_with_all_fields() {
        let start_date = Some("2025-01-01".to_string());
        let end_date = Some("2025-12-31".to_string());
        let vacation_rules = Some(VacationRulesManifest {
            max_concurrent_vacations: Some(5),
            allow_layoff_vacations: Some(true),
            require_layoff_vacation_period: Some(false),
            layoff_periods: None,
        });

        let spec = ProjectSpec {
            name: "My Project Name".to_string(),
            description: Some("A cool project".to_string()),
            timezone: None,
            start_date: start_date.clone(),
            end_date: end_date.clone(),
            status: ProjectStatusManifest::InProgress,
            vacation_rules,
        };

        let serialized_spec = serde_yaml::to_string(&spec).unwrap();

        let expected_yaml = format!(
            r#"name: My Project Name
description: A cool project
timezone: null
startDate: {}
endDate: {}
status: InProgress
vacationRules:
  maxConcurrentVacations: 5
  allowLayoffVacations: true
  requireLayoffVacationPeriod: false"#,
            start_date.unwrap_or_else(|| "".to_string()),
            end_date.unwrap_or_else(|| "".to_string())
        );

        assert_eq!(serialized_spec.trim(), expected_yaml.trim());
    }
}
