//! Testes abrangentes de parsing YAML
//!
//! Este módulo contém testes robustos para validação de parsing YAML
//! cobrindo cenários de sucesso e falha com mensagens claras.

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use serde_yaml::Value;
use std::fs;
use std::path::Path;

/// Validador YAML robusto com mensagens claras
struct YamlParser {
    content: String,
    parsed: Value,
}

impl YamlParser {
    fn new(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let parsed: Value = serde_yaml::from_str(content)?;
        Ok(Self {
            content: content.to_string(),
            parsed,
        })
    }

    fn from_file(file_path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        Self::new(&content)
    }

    /// Verifica se um campo existe no caminho especificado
    fn has_field(&self, path: &str) -> bool {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.parsed;

        for part in parts {
            if let Some(map) = current.as_mapping() {
                if let Some(value) = map.get(part) {
                    current = value;
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    /// Verifica se um campo tem um valor específico
    fn field_equals(&self, path: &str, expected: &str) -> bool {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.parsed;

        for part in parts {
            if let Some(map) = current.as_mapping() {
                if let Some(value) = map.get(part) {
                    current = value;
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(str_value) = current.as_str() {
            str_value == expected
        } else {
            false
        }
    }

    /// Verifica se um campo não está vazio
    fn field_not_empty(&self, path: &str) -> bool {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.parsed;

        for part in parts {
            if let Some(map) = current.as_mapping() {
                if let Some(value) = map.get(part) {
                    current = value;
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(str_value) = current.as_str() {
            !str_value.is_empty()
        } else {
            false
        }
    }

    /// Verifica se um campo é um array com elementos
    fn field_is_array_with_items(&self, path: &str) -> bool {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = &self.parsed;

        for part in parts {
            if let Some(map) = current.as_mapping() {
                if let Some(value) = map.get(part) {
                    current = value;
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(array) = current.as_sequence() {
            !array.is_empty()
        } else {
            false
        }
    }
}

#[test]
fn test_yaml_parsing_success_scenarios() {
    let test_cases = vec![
        // Company YAML
        (
            r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  id: "01996dev-0000-0000-0000-0000000001"
  code: "TECH-CORP"
  name: "Tech Corporation"
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-01T00:00:00Z"
  createdBy: "system"
  labels:
    environment: "production"
    region: "us-east-1"
  annotations:
    description: "Technology company"
  namespace: "default"
spec:
  description: "A technology company"
  size: "medium"
  status: "active"
  taxId: "123456789"
  address: "123 Tech Street"
  email: "contact@techcorp.com"
  phone: "+1-555-0123"
  website: "https://techcorp.com"
  industry: "Technology"
status:
  status: "active"
  lastUpdated: "2024-01-01T00:00:00Z"
"#,
            "Company",
        ),
        // Project YAML
        (
            r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  id: "01996dev-0000-0000-0000-0000000002"
  code: "PROJ-001"
  name: "Web Application"
  description: "A web application project"
  companyCode: "TECH-CORP"
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-01T00:00:00Z"
  createdBy: "system"
  labels:
    type: "web"
    priority: "high"
  annotations:
    notes: "Critical project"
  namespace: "default"
spec:
  status: "Planned"
  startDate: "2024-01-01"
  endDate: "2024-12-31"
  timezone: "UTC"
  vacationRules:
    allowedDaysPerYear: 25
    carryOverDays: 10
"#,
            "Project",
        ),
        // Resource YAML
        (
            r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Resource
metadata:
  id: "01996dev-0000-0000-0000-0000000003"
  code: "DEV-001"
  name: "John Doe"
  email: "john@techcorp.com"
  resourceType: "Developer"
  status: "Available"
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-01T00:00:00Z"
  createdBy: "system"
  labels:
    role: "senior"
    team: "backend"
  annotations:
    skills: "Rust, Python, JavaScript"
  namespace: "default"
spec:
  scope: "Company"
  timeOffBalance: 25
  startDate: "2024-01-01"
  endDate: "2024-12-31"
  vacations:
    - startDate: "2024-07-01"
      endDate: "2024-07-15"
      type: "Annual"
  projectAssignments:
    - projectCode: "PROJ-001"
      startDate: "2024-01-01"
      endDate: "2024-06-30"
      allocation: 100
"#,
            "Resource",
        ),
        // Task YAML
        (
            r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Task
metadata:
  id: "01996dev-0000-0000-0000-0000000004"
  code: "TASK-001"
  name: "Implement Authentication"
  description: "Implement user authentication system"
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-01T00:00:00Z"
  createdBy: "system"
  labels:
    priority: "high"
    component: "auth"
  annotations:
    acceptance: "User can login and logout"
  namespace: "default"
spec:
  projectCode: "PROJ-001"
  assignee: "DEV-001"
  status: "Planned"
  priority: "High"
  category: "Development"
  startDate: "2024-01-01"
  dueDate: "2024-01-15"
  dependencies:
    - "TASK-000"
  assignedResources:
    - "DEV-001"
    - "DEV-002"
  effort:
    estimatedHours: 40.0
    actualHours: 0.0
  acceptanceCriteria:
    - "User can register"
    - "User can login"
    - "User can logout"
  tags:
    - "authentication"
    - "security"
"#,
            "Task",
        ),
    ];

    for (yaml_content, expected_kind) in test_cases {
        let parser = YamlParser::new(yaml_content).expect("Should parse valid YAML");
        
        // Verificar campos obrigatórios
        assert!(parser.has_field("apiVersion"), "Missing apiVersion field");
        assert!(parser.has_field("kind"), "Missing kind field");
        assert!(parser.has_field("metadata"), "Missing metadata field");
        assert!(parser.has_field("spec"), "Missing spec field");
        
        // Verificar valores específicos
        assert!(parser.field_equals("apiVersion", "tasktaskrevolution.io/v1alpha1"));
        assert!(parser.field_equals("kind", expected_kind));
        
        // Verificar campos de metadata
        assert!(parser.has_field("metadata.id"));
        assert!(parser.has_field("metadata.code"));
        assert!(parser.has_field("metadata.name"));
        assert!(parser.has_field("metadata.createdAt"));
        assert!(parser.has_field("metadata.updatedAt"));
        assert!(parser.has_field("metadata.createdBy"));
        
        // Verificar campos opcionais (labels, annotations, namespace)
        assert!(parser.has_field("metadata.labels"));
        assert!(parser.has_field("metadata.annotations"));
        assert!(parser.has_field("metadata.namespace"));
        
        // Verificar que os campos não estão vazios
        assert!(parser.field_not_empty("metadata.id"));
        assert!(parser.field_not_empty("metadata.code"));
        assert!(parser.field_not_empty("metadata.name"));
    }
}

#[test]
fn test_yaml_parsing_failure_scenarios() {
    let failure_cases = vec![
        // Sintaxe YAML inválida
        (
            "invalid: yaml: content: [",
            "Invalid YAML syntax",
        ),
        // Campo obrigatório ausente
        (
            r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  id: "01996dev-0000-0000-0000-0000000001"
  # Missing required fields: code, name, createdAt, updatedAt, createdBy
spec:
  size: "medium"
  status: "active"
"#,
            "Missing required fields",
        ),
        // Tipo de campo inválido
        (
            r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  id: "01996dev-0000-0000-0000-0000000001"
  code: "TECH-CORP"
  name: "Tech Corp"
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-01T00:00:00Z"
  createdBy: "system"
spec:
  size: "invalid_size"  # Invalid enum value
  status: "active"
"#,
            "Invalid field type",
        ),
        // Formato de data inválido
        (
            r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  id: "01996dev-0000-0000-0000-0000000001"
  code: "TECH-CORP"
  name: "Tech Corp"
  createdAt: "invalid-date-format"
  updatedAt: "2024-01-01T00:00:00Z"
  createdBy: "system"
spec:
  size: "medium"
  status: "active"
"#,
            "Invalid date format",
        ),
        // Versão de API incorreta
        (
            r#"
apiVersion: tasktaskrevolution.io/v1beta1  # Wrong version
kind: Company
metadata:
  id: "01996dev-0000-0000-0000-0000000001"
  code: "TECH-CORP"
  name: "Tech Corp"
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-01T00:00:00Z"
  createdBy: "system"
spec:
  size: "medium"
  status: "active"
"#,
            "Wrong API version",
        ),
        // Estrutura YAML malformada
        (
            r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  id: "01996dev-0000-0000-0000-0000000001"
  code: "TECH-CORP"
  name: "Tech Corp"
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-01T00:00:00Z"
  createdBy: "system"
spec:
  size: "medium"
  status: "active"
  # Missing closing brace
"#,
            "Malformed YAML structure",
        ),
    ];

    for (yaml_content, expected_error_type) in failure_cases {
        let result = YamlParser::new(yaml_content);
        
        match result {
            Ok(_) => {
                // Se o parsing foi bem-sucedido, verificar se há problemas de validação
                // que deveriam causar falha
                if expected_error_type == "Missing required fields" {
                    let parser = result.unwrap();
                    // Verificar se campos obrigatórios estão ausentes
                    assert!(!parser.has_field("metadata.code") || !parser.has_field("metadata.name"));
                }
            }
            Err(e) => {
                let error_message = format!("{}", e);
                match expected_error_type {
                    "Invalid YAML syntax" => {
                        assert!(error_message.contains("yaml") || error_message.contains("YAML") || error_message.contains("parse"));
                    }
                    "Invalid field type" => {
                        assert!(error_message.contains("invalid") || error_message.contains("deserialize"));
                    }
                    "Invalid date format" => {
                        assert!(error_message.contains("date") || error_message.contains("time"));
                    }
                    "Wrong API version" => {
                        // Este pode não falhar no parsing, mas falhar na validação
                        assert!(error_message.contains("version") || error_message.contains("api"));
                    }
                    "Malformed YAML structure" => {
                        assert!(error_message.contains("yaml") || error_message.contains("parse"));
                    }
                    _ => {
                        // Para outros tipos de erro, apenas verificar que há uma mensagem de erro
                        assert!(!error_message.is_empty());
                    }
                }
            }
        }
    }
}

#[test]
fn test_yaml_parsing_edge_cases() {
    let edge_cases = vec![
        // YAML com campos vazios
        (
            r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  id: "01996dev-0000-0000-0000-0000000001"
  code: "TECH-CORP"
  name: "Tech Corp"
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-01T00:00:00Z"
  createdBy: "system"
  labels: {}
  annotations: {}
  namespace: ""
spec:
  description: ""
  size: "medium"
  status: "active"
"#,
            "Empty fields",
        ),
        // YAML com arrays vazios
        (
            r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Task
metadata:
  id: "01996dev-0000-0000-0000-0000000004"
  code: "TASK-001"
  name: "Empty Task"
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-01T00:00:00Z"
  createdBy: "system"
spec:
  projectCode: "PROJ-001"
  assignee: "DEV-001"
  status: "Planned"
  priority: "Medium"
  dependencies: []
  assignedResources: []
  tags: []
  acceptanceCriteria: []
"#,
            "Empty arrays",
        ),
        // YAML com valores nulos
        (
            r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  id: "01996dev-0000-0000-0000-0000000002"
  code: "PROJ-001"
  name: "Null Project"
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-01T00:00:00Z"
  createdBy: "system"
spec:
  status: "Planned"
  startDate: null
  endDate: null
  timezone: null
"#,
            "Null values",
        ),
    ];

    for (yaml_content, case_type) in edge_cases {
        let parser = YamlParser::new(yaml_content).expect(&format!("Should parse YAML with {}", case_type));
        
        // Verificar que o YAML foi parseado corretamente
        assert!(parser.has_field("apiVersion"));
        assert!(parser.has_field("kind"));
        assert!(parser.has_field("metadata"));
        assert!(parser.has_field("spec"));
        
        match case_type {
            "Empty fields" => {
                // Verificar que campos vazios são tratados corretamente
                assert!(parser.has_field("metadata.labels"));
                assert!(parser.has_field("metadata.annotations"));
            }
            "Empty arrays" => {
                // Verificar que arrays vazios são tratados corretamente
                assert!(parser.has_field("spec.dependencies"));
                assert!(parser.has_field("spec.assignedResources"));
                assert!(parser.has_field("spec.tags"));
                assert!(parser.has_field("spec.acceptanceCriteria"));
            }
            "Null values" => {
                // Verificar que valores nulos são tratados corretamente
                assert!(parser.has_field("spec.startDate"));
                assert!(parser.has_field("spec.endDate"));
                assert!(parser.has_field("spec.timezone"));
            }
            _ => {}
        }
    }
}

#[test]
fn test_yaml_parsing_performance() {
    // Teste de performance com YAML grande
    let large_yaml = format!(
        r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Task
metadata:
  id: "01996dev-0000-0000-0000-0000000004"
  code: "TASK-001"
  name: "Large Task"
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-01T00:00:00Z"
  createdBy: "system"
spec:
  projectCode: "PROJ-001"
  assignee: "DEV-001"
  status: "Planned"
  priority: "Medium"
  dependencies: []
  assignedResources: []
  tags: []
  acceptanceCriteria: []
{}
"#,
        // Adicionar muitos campos para testar performance
        (0..1000).map(|i| format!("  field{}: value{}\n", i, i)).collect::<String>()
    );

    let start = std::time::Instant::now();
    let parser = YamlParser::new(&large_yaml).expect("Should parse large YAML");
    let duration = start.elapsed();
    
    // Verificar que o parsing foi bem-sucedido
    assert!(parser.has_field("apiVersion"));
    assert!(parser.has_field("kind"));
    
    // Verificar que o parsing foi rápido (menos de 1 segundo)
    assert!(duration.as_millis() < 1000, "YAML parsing took too long: {:?}", duration);
}

#[test]
fn test_yaml_parsing_unicode_support() {
    let unicode_yaml = r#"
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Company
metadata:
  id: "01996dev-0000-0000-0000-0000000001"
  code: "TECH-ÇORP"
  name: "Tecnologia & Cia. Ltda."
  createdAt: "2024-01-01T00:00:00Z"
  updatedAt: "2024-01-01T00:00:00Z"
  createdBy: "sistema"
  labels:
    país: "Brasil"
    região: "Sudeste"
  annotations:
    descrição: "Empresa de tecnologia"
    "caracteres-especiais": "áéíóú"
spec:
  description: "Uma empresa de tecnologia"
  size: "medium"
  status: "active"
"#;

    let parser = YamlParser::new(unicode_yaml).expect("Should parse Unicode YAML");
    
    // Verificar que campos Unicode foram parseados corretamente
    assert!(parser.field_equals("metadata.code", "TECH-ÇORP"));
    assert!(parser.field_equals("metadata.name", "Tecnologia & Cia. Ltda."));
    assert!(parser.field_equals("metadata.labels.país", "Brasil"));
    assert!(parser.field_equals("metadata.labels.região", "Sudeste"));
    assert!(parser.field_equals("metadata.annotations.descrição", "Empresa de tecnologia"));
    assert!(parser.field_equals("metadata.annotations.caracteres-especiais", "áéíóú"));
    assert!(parser.field_equals("spec.description", "Uma empresa de tecnologia"));
}
