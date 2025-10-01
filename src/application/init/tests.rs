use super::*;
use crate::domain::shared::errors::{AppError, AppErrorKind};
use crate::domain::company_settings::{Config, WorkDay};
use crate::infrastructure::persistence::config_repository::ConfigRepository;
use crate::domain::shared::errors::{DomainError, DomainResult};
use crate::infrastructure::persistence::manifests::config_manifest::ConfigManifest;
use chrono::{NaiveTime, Utc};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

// Mock repository for testing
struct MockConfigRepository {
    should_fail: bool,
    saved_config: RefCell<Option<Config>>,
}

impl MockConfigRepository {
    fn new() -> Self {
        Self {
            should_fail: false,
            saved_config: RefCell::new(None),
        }
    }

    fn with_failure() -> Self {
        Self {
            should_fail: true,
            saved_config: RefCell::new(None),
        }
    }
}

impl ConfigRepository for MockConfigRepository {
    fn save(&self, _config: Config, _path: &Path) -> DomainResult<()> {
        if self.should_fail {
            return Err(DomainError::IoError {
                operation: "save".to_string(),
                details: "Database connection failed".to_string(),
            });
        }
        Ok(())
    }

    fn create_repository_dir(&self, _path: &Path) -> DomainResult<()> {
        Ok(())
    }

    fn load(&self) -> DomainResult<(Config, PathBuf)> {
        self.saved_config.borrow().clone().map(|c| {
            (c, PathBuf::from("/tmp"))
        }).ok_or(DomainError::ValidationError {
            field: "config".to_string(),
            message: "Configuration missing".to_string(),
        })
    }
}

#[test]
fn test_init_manager_success() {
    // Arrange
    let mock_repo = MockConfigRepository::new();
    let use_case = InitManagerUseCase::new(Box::new(mock_repo));

    let init_data = InitManagerData {
        name: "João Silva".to_string(),
        email: "joao.silva@consultoria.com".to_string(),
        company_name: "TechConsulting Ltda".to_string(),
        timezone: "America/Sao_Paulo".to_string(),
        work_hours_start: "08:00".to_string(),
        work_hours_end: "18:00".to_string(),
        work_days: "monday,tuesday,wednesday,thursday,friday".to_string(),
    };

    // Act
    let result = use_case.execute(init_data);

    // Assert
    assert!(result.is_ok());
    let config = result.unwrap();

    assert_eq!(config.manager_name, "João Silva");
    assert_eq!(config.manager_email, "joao.silva@consultoria.com");
    assert_eq!(config.company_name, Some("TechConsulting Ltda".to_string()));
    assert_eq!(config.default_timezone, "America/Sao_Paulo");
    assert_eq!(config.work_hours_start, Some("08:00".to_string()));
    assert_eq!(config.work_hours_end, Some("18:00".to_string()));
}

#[test]
fn test_init_manager_invalid_email() {
    // Arrange
    let mock_repo = MockConfigRepository::new();
    let use_case = InitManagerUseCase::new(Box::new(mock_repo));

    let init_data = InitManagerData {
        name: "João Silva".to_string(),
        email: "email-invalido".to_string(),
        company_name: "TechConsulting Ltda".to_string(),
        timezone: "America/Sao_Paulo".to_string(),
        work_hours_start: "08:00".to_string(),
        work_hours_end: "18:00".to_string(),
        work_days: "monday,tuesday,wednesday,thursday,friday".to_string(),
    };

    // Act
    let result = use_case.execute(init_data);

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error.kind(), AppErrorKind::ValidationError { .. }));
}

#[test]
fn test_init_manager_invalid_timezone() {
    // Arrange
    let mock_repo = MockConfigRepository::new();
    let use_case = InitManagerUseCase::new(Box::new(mock_repo));

    let init_data = InitManagerData {
        name: "João Silva".to_string(),
        email: "joao.silva@consultoria.com".to_string(),
        company_name: "TechConsulting Ltda".to_string(),
        timezone: "Timezone/Invalido".to_string(),
        work_hours_start: "08:00".to_string(),
        work_hours_end: "18:00".to_string(),
        work_days: "monday,tuesday,wednesday,thursday,friday".to_string(),
    };

    // Act
    let result = use_case.execute(init_data);

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error.kind(), AppErrorKind::ValidationError { .. }));
}

#[test]
fn test_init_manager_invalid_work_hours() {
    // Arrange
    let mock_repo = MockConfigRepository::new();
    let use_case = InitManagerUseCase::new(Box::new(mock_repo));

    let init_data = InitManagerData {
        name: "João Silva".to_string(),
        email: "joao.silva@consultoria.com".to_string(),
        company_name: "TechConsulting Ltda".to_string(),
        timezone: "America/Sao_Paulo".to_string(),
        work_hours_start: "18:00".to_string(),
        work_hours_end: "08:00".to_string(), // End before start
        work_days: "monday,tuesday,wednesday,thursday,friday".to_string(),
    };

    // Act
    let result = use_case.execute(init_data);

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error.kind(), AppErrorKind::ValidationError { .. }));
}

#[test]
fn test_init_manager_empty_name() {
    // Arrange
    let mock_repo = MockConfigRepository::new();
    let use_case = InitManagerUseCase::new(Box::new(mock_repo));

    let init_data = InitManagerData {
        name: "".to_string(),
        email: "joao.silva@consultoria.com".to_string(),
        company_name: "TechConsulting Ltda".to_string(),
        timezone: "America/Sao_Paulo".to_string(),
        work_hours_start: "08:00".to_string(),
        work_hours_end: "18:00".to_string(),
        work_days: "monday,tuesday,wednesday,thursday,friday".to_string(),
    };

    // Act
    let result = use_case.execute(init_data);

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error.kind(), AppErrorKind::ValidationError { .. }));
}

#[test]
fn test_init_manager_empty_company_name() {
    // Arrange
    let mock_repo = MockConfigRepository::new();
    let use_case = InitManagerUseCase::new(Box::new(mock_repo));

    let init_data = InitManagerData {
        name: "João Silva".to_string(),
        email: "joao.silva@consultoria.com".to_string(),
        company_name: "".to_string(),
        timezone: "America/Sao_Paulo".to_string(),
        work_hours_start: "08:00".to_string(),
        work_hours_end: "18:00".to_string(),
        work_days: "monday,tuesday,wednesday,thursday,friday".to_string(),
    };

    // Act
    let result = use_case.execute(init_data);

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error.kind(), AppErrorKind::ValidationError { .. }));
}

#[test]
fn test_init_manager_repository_error() {
    // Arrange
    let mock_repo = MockConfigRepository::with_failure();
    let use_case = InitManagerUseCase::new(Box::new(mock_repo));

    let init_data = InitManagerData {
        name: "João Silva".to_string(),
        email: "joao.silva@consultoria.com".to_string(),
        company_name: "TechConsulting Ltda".to_string(),
        timezone: "America/Sao_Paulo".to_string(),
        work_hours_start: "08:00".to_string(),
        work_hours_end: "18:00".to_string(),
        work_days: "monday,tuesday,wednesday,thursday,friday".to_string(),
    };

    // Act
    let result = use_case.execute(init_data);

    // Assert
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error.kind(), AppErrorKind::PersistenceError { .. }));
}

#[test]
fn test_init_manager_different_timezones() {
    let test_cases = vec![
        "UTC",
        "America/New_York",
        "Europe/London",
        "Asia/Tokyo",
    ];

    for timezone in test_cases {
        // Arrange
        let mock_repo = MockConfigRepository::new();
        let use_case = InitManagerUseCase::new(Box::new(mock_repo));

        let init_data = InitManagerData {
            name: "João Silva".to_string(),
            email: "joao.silva@consultoria.com".to_string(),
            company_name: "TechConsulting Ltda".to_string(),
            timezone: timezone.to_string(),
            work_hours_start: "08:00".to_string(),
            work_hours_end: "18:00".to_string(),
            work_days: "monday,tuesday,wednesday,thursday,friday".to_string(),
        };

        // Act
        let result = use_case.execute(init_data);

        // Assert
        assert!(result.is_ok(), "Failed for timezone: {}", timezone);
        let config = result.unwrap();
        assert_eq!(config.default_timezone, timezone, "Timezone mismatch for: {}", timezone);
    }
}

#[test]
fn test_init_manager_work_hours_edge_cases() {
    let test_cases = vec![
        ("00:00", "23:59"), // Midnight to almost midnight
        ("09:30", "17:30"), // Half hours
        ("12:00", "13:00"), // Lunch break
    ];

    for (start, end) in test_cases {
        // Arrange
        let mock_repo = MockConfigRepository::new();
        let use_case = InitManagerUseCase::new(Box::new(mock_repo));

        let init_data = InitManagerData {
            name: "João Silva".to_string(),
            email: "joao.silva@consultoria.com".to_string(),
            company_name: "TechConsulting Ltda".to_string(),
            timezone: "UTC".to_string(),
            work_hours_start: start.to_string(),
            work_hours_end: end.to_string(),
            work_days: "monday,tuesday,wednesday,thursday,friday".to_string(),
        };

        // Act
        let result = use_case.execute(init_data);

        // Assert
        assert!(result.is_ok(), "Failed for work hours: {} - {}", start, end);
        let config = result.unwrap();

        assert_eq!(config.work_hours_start, Some(start.to_string()));
        assert_eq!(config.work_hours_end, Some(end.to_string()));
    }
}
