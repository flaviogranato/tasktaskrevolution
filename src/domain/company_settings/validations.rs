#![allow(dead_code)]

use crate::domain::shared::errors::{DomainError, DomainResult};

/// Validações de domínio para configurações da empresa
pub struct CompanySettingsValidator;

impl CompanySettingsValidator {
    /// Valida o nome do gerente
    pub fn validate_manager_name(name: &str) -> DomainResult<()> {
        if name.trim().is_empty() {
            return Err(DomainError::ConfigurationInvalid {
                field: "manager_name".to_string(),
                value: name.to_string(),
                reason: "Nome do gerente não pode estar vazio".to_string(),
            });
        }

        if name.len() < 2 {
            return Err(DomainError::ConfigurationInvalid {
                field: "manager_name".to_string(),
                value: name.to_string(),
                reason: "Nome do gerente deve ter pelo menos 2 caracteres".to_string(),
            });
        }

        if name.len() > 100 {
            return Err(DomainError::ConfigurationInvalid {
                field: "manager_name".to_string(),
                value: name.to_string(),
                reason: "Nome do gerente não pode exceder 100 caracteres".to_string(),
            });
        }

        // Validar se contém apenas caracteres válidos (letras, espaços, hífens e acentos)
        if !name
            .chars()
            .all(|c| c.is_alphabetic() || c.is_whitespace() || c == '-' || c == '\'')
        {
            return Err(DomainError::ConfigurationInvalid {
                field: "manager_name".to_string(),
                value: name.to_string(),
                reason: "Nome do gerente contém caracteres inválidos".to_string(),
            });
        }

        Ok(())
    }

    /// Valida o email do gerente
    pub fn validate_manager_email(email: &str) -> DomainResult<()> {
        if email.trim().is_empty() {
            return Err(DomainError::ConfigurationInvalid {
                field: "manager_email".to_string(),
                value: email.to_string(),
                reason: "Email do gerente não pode estar vazio".to_string(),
            });
        }

        // Validação básica de formato de email
        if !email.contains('@') || !email.contains('.') {
            return Err(DomainError::ConfigurationInvalid {
                field: "manager_email".to_string(),
                value: email.to_string(),
                reason: "Formato de email inválido".to_string(),
            });
        }

        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return Err(DomainError::ConfigurationInvalid {
                field: "manager_email".to_string(),
                value: email.to_string(),
                reason: "Formato de email inválido".to_string(),
            });
        }

        let local_part = parts[0];
        let domain_part = parts[1];

        if local_part.is_empty() || domain_part.is_empty() {
            return Err(DomainError::ConfigurationInvalid {
                field: "manager_email".to_string(),
                value: email.to_string(),
                reason: "Partes do email não podem estar vazias".to_string(),
            });
        }

        if local_part.len() > 64 || domain_part.len() > 253 {
            return Err(DomainError::ConfigurationInvalid {
                field: "manager_email".to_string(),
                value: email.to_string(),
                reason: "Email muito longo".to_string(),
            });
        }

        Ok(())
    }

    /// Valida o fuso horário padrão
    pub fn validate_default_timezone(timezone: &str) -> DomainResult<()> {
        if timezone.trim().is_empty() {
            return Err(DomainError::ConfigurationInvalid {
                field: "default_timezone".to_string(),
                value: timezone.to_string(),
                reason: "Fuso horário padrão não pode estar vazio".to_string(),
            });
        }

        // Lista de fusos horários válidos comuns
        let valid_timezones = [
            "UTC",
            "GMT",
            "EST",
            "CST",
            "MST",
            "PST",
            "America/New_York",
            "America/Chicago",
            "America/Denver",
            "America/Los_Angeles",
            "America/Sao_Paulo",
            "America/Argentina/Buenos_Aires",
            "America/Mexico_City",
            "Europe/London",
            "Europe/Paris",
            "Europe/Berlin",
            "Europe/Rome",
            "Europe/Madrid",
            "Asia/Shanghai",
            "Asia/Singapore",
            "Asia/Dubai",
            "Australia/Sydney",
            "Australia/Melbourne",
        ];

        if !valid_timezones.contains(&timezone) {
            return Err(DomainError::ConfigurationInvalid {
                field: "default_timezone".to_string(),
                value: timezone.to_string(),
                reason: format!("Fuso horário '{}' não é suportado", timezone),
            });
        }

        Ok(())
    }

    /// Valida todas as configurações da empresa
    pub fn validate_all_config(manager_name: &str, manager_email: &str, default_timezone: &str) -> DomainResult<()> {
        Self::validate_manager_name(manager_name)?;
        Self::validate_manager_email(manager_email)?;
        Self::validate_default_timezone(default_timezone)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_manager_name_success() {
        assert!(CompanySettingsValidator::validate_manager_name("John Doe").is_ok());
        assert!(CompanySettingsValidator::validate_manager_name("Maria José").is_ok());
        assert!(CompanySettingsValidator::validate_manager_name("Jean-Pierre").is_ok());
    }

    #[test]
    fn test_validate_manager_name_empty() {
        let result = CompanySettingsValidator::validate_manager_name("");
        assert!(
            matches!(result, Err(DomainError::ConfigurationInvalid { field, reason, value: _ })
            if field == "manager_name" && reason.contains("vazio"))
        );
    }

    #[test]
    fn test_validate_manager_name_too_short() {
        let result = CompanySettingsValidator::validate_manager_name("A");
        assert!(
            matches!(result, Err(DomainError::ConfigurationInvalid { field, reason, value: _ })
            if field == "manager_name" && reason.contains("2 caracteres"))
        );
    }

    #[test]
    fn test_validate_manager_name_too_long() {
        let long_name = "A".repeat(101);
        let result = CompanySettingsValidator::validate_manager_name(&long_name);
        assert!(
            matches!(result, Err(DomainError::ConfigurationInvalid { field, reason, value: _ })
            if field == "manager_name" && reason.contains("100 caracteres"))
        );
    }

    #[test]
    fn test_validate_manager_name_invalid_chars() {
        let result = CompanySettingsValidator::validate_manager_name("John123");
        assert!(
            matches!(result, Err(DomainError::ConfigurationInvalid { field, reason, value: _ })
            if field == "manager_name" && reason.contains("caracteres inválidos"))
        );
    }

    #[test]
    fn test_validate_manager_email_success() {
        assert!(CompanySettingsValidator::validate_manager_email("john@example.com").is_ok());
        assert!(CompanySettingsValidator::validate_manager_email("maria.jose@empresa.com.br").is_ok());
        assert!(CompanySettingsValidator::validate_manager_email("user+tag@domain.org").is_ok());
    }

    #[test]
    fn test_validate_manager_email_empty() {
        let result = CompanySettingsValidator::validate_manager_email("");
        assert!(
            matches!(result, Err(DomainError::ConfigurationInvalid { field, reason, value: _ })
            if field == "manager_email" && reason.contains("vazio"))
        );
    }

    #[test]
    fn test_validate_manager_email_no_at() {
        let result = CompanySettingsValidator::validate_manager_email("johnexample.com");
        assert!(
            matches!(result, Err(DomainError::ConfigurationInvalid { field, reason, value: _ })
            if field == "manager_email" && reason.contains("Formato de email inválido"))
        );
    }

    #[test]
    fn test_validate_manager_email_no_dot() {
        let result = CompanySettingsValidator::validate_manager_email("john@example");
        assert!(
            matches!(result, Err(DomainError::ConfigurationInvalid { field, reason, value: _ })
            if field == "manager_email" && reason.contains("Formato de email inválido"))
        );
    }

    #[test]
    fn test_validate_default_timezone_success() {
        assert!(CompanySettingsValidator::validate_default_timezone("UTC").is_ok());
        assert!(CompanySettingsValidator::validate_default_timezone("America/Sao_Paulo").is_ok());
        assert!(CompanySettingsValidator::validate_default_timezone("Europe/London").is_ok());
    }

    #[test]
    fn test_validate_default_timezone_empty() {
        let result = CompanySettingsValidator::validate_default_timezone("");
        assert!(
            matches!(result, Err(DomainError::ConfigurationInvalid { field, reason, value: _ })
            if field == "default_timezone" && reason.contains("vazio"))
        );
    }

    #[test]
    fn test_validate_default_timezone_invalid() {
        let result = CompanySettingsValidator::validate_default_timezone("Invalid/Timezone");
        assert!(
            matches!(result, Err(DomainError::ConfigurationInvalid { field, reason, value: _ })
            if field == "default_timezone" && reason.contains("não é suportado"))
        );
    }

    #[test]
    fn test_validate_all_config_success() {
        let result = CompanySettingsValidator::validate_all_config("John Doe", "john@example.com", "UTC");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_all_config_failure() {
        let result = CompanySettingsValidator::validate_all_config("", "invalid-email", "Invalid/Timezone");
        assert!(result.is_err());
    }
}
