use crate::domain::shared::errors::DomainError;
use crate::domain::company_settings::errors::CompanySettingsError;
use crate::domain::company_settings::validations::CompanySettingsValidator;
use crate::domain::company_settings::config::Config;

/// Regras de negócio para configurações da empresa
pub struct CompanySettingsBusinessRules;

impl CompanySettingsBusinessRules {
    /// Aplica todas as regras de negócio para criar uma nova configuração
    pub fn apply_creation_rules(
        manager_name: &str,
        manager_email: &str,
        default_timezone: &str,
    ) -> Result<Config, CompanySettingsError> {
        // 1. Aplicar regras de negócio específicas primeiro
        Self::apply_name_business_rules(manager_name)?;
        Self::apply_email_business_rules(manager_email)?;
        Self::apply_timezone_business_rules(default_timezone)?;

        // 2. Validar dados de entrada
        CompanySettingsValidator::validate_all_config(
            manager_name,
            manager_email,
            default_timezone,
        )?;

        // 3. Criar e retornar a configuração
        Ok(Config::new(
            manager_name.to_string(),
            manager_email.to_string(),
            default_timezone.to_string(),
        ))
    }

    /// Aplica regras de negócio específicas para o nome
    fn apply_name_business_rules(name: &str) -> Result<(), CompanySettingsError> {
        // Regra: Nome não pode conter apenas espaços
        if name.trim().is_empty() {
            return Err(CompanySettingsError::ConfigurationInvalid {
                field: "manager_name".to_string(),
                value: name.to_string(),
                reason: "Nome do gerente não pode conter apenas espaços".to_string(),
            });
        }

        // Regra: Nome deve ter pelo menos uma palavra com 2+ caracteres
        let words: Vec<&str> = name.split_whitespace().collect();
        let has_valid_word = words.iter().any(|word| word.len() >= 2);
        
        if !has_valid_word {
            return Err(CompanySettingsError::ConfigurationInvalid {
                field: "manager_name".to_string(),
                value: name.to_string(),
                reason: "Nome do gerente deve ter pelo menos uma palavra com 2+ caracteres".to_string(),
            });
        }

        // Regra: Nome não pode começar ou terminar com hífen ou apóstrofo
        if name.starts_with('-') || name.starts_with('\'') || 
           name.ends_with('-') || name.ends_with('\'') {
            return Err(CompanySettingsError::ConfigurationInvalid {
                field: "manager_name".to_string(),
                value: name.to_string(),
                reason: "Nome do gerente não pode começar ou terminar com hífen ou apóstrofo".to_string(),
            });
        }

        Ok(())
    }

    /// Aplica regras de negócio específicas para o email
    fn apply_email_business_rules(email: &str) -> Result<(), CompanySettingsError> {
        // Regra: Email deve ser único (simulado - em produção seria verificado no banco)
        if email == "admin@system.local" {
            return Err(CompanySettingsError::ConfigurationInvalid {
                field: "manager_email".to_string(),
                value: email.to_string(),
                reason: "Email 'admin@system.local' é reservado para o sistema".to_string(),
            });
        }

        // Regra: Email não pode ser muito genérico
        let generic_emails = ["test@example.com", "admin@company.com", "user@domain.com"];
        if generic_emails.contains(&email) {
            return Err(CompanySettingsError::ConfigurationInvalid {
                field: "manager_email".to_string(),
                value: email.to_string(),
                reason: "Email muito genérico, use um email específico da empresa".to_string(),
            });
        }

        // Regra: Email deve ter domínio válido (não pode ser localhost, etc.)
        let invalid_domains = ["localhost", "127.0.0.1", "::1", "0.0.0.0"];
        if let Some(domain) = email.split('@').nth(1) {
            if invalid_domains.contains(&domain) {
                return Err(CompanySettingsError::ConfigurationInvalid {
                    field: "manager_email".to_string(),
                    value: email.to_string(),
                    reason: "Domínio de email inválido".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Aplica regras de negócio específicas para o fuso horário
    fn apply_timezone_business_rules(timezone: &str) -> Result<(), CompanySettingsError> {
        // Regra: Fuso horário deve ser apropriado para o contexto da empresa
        let recommended_timezones = [
            "America/Sao_Paulo", "America/New_York", "Europe/London", "UTC"
        ];

        if !recommended_timezones.contains(&timezone) {
            // Apenas um warning, não um erro
            // Em produção, isso poderia ser logado
        }

        // Regra: Não permitir fusos horários muito extremos para empresas
        let extreme_timezones = ["Asia/Tokyo", "Pacific/Auckland"];
        if extreme_timezones.contains(&timezone) {
            return Err(CompanySettingsError::ConfigurationInvalid {
                field: "default_timezone".to_string(),
                value: timezone.to_string(),
                reason: "Fuso horário muito extremo para o contexto da empresa".to_string(),
            });
        }

        Ok(())
    }

    /// Aplica regras de negócio para atualização de configurações
    pub fn apply_update_rules(
        current_config: &Config,
        new_manager_name: Option<&str>,
        new_manager_email: Option<&str>,
        new_default_timezone: Option<&str>,
    ) -> Result<Config, CompanySettingsError> {
        let manager_name = new_manager_name.unwrap_or(&current_config.manager_name);
        let manager_email = new_manager_email.unwrap_or(&current_config.manager_email);
        let default_timezone = new_default_timezone.unwrap_or(&current_config.default_timezone);

        // Aplicar regras de criação
        Self::apply_creation_rules(manager_name, manager_email, default_timezone)
    }

    /// Valida se uma configuração pode ser removida
    pub fn can_remove_config(config: &Config) -> Result<bool, CompanySettingsError> {
        // Regra: Configuração não pode ser removida se for a única configuração ativa
        // Simulado - em produção seria verificado no banco
        if config.manager_email == "admin@system.local" {
            return Err(CompanySettingsError::OperationNotAllowed {
                operation: "remove".to_string(),
                reason: "Configuração do sistema não pode ser removida".to_string(),
            });
        }

        Ok(true)
    }

    /// Aplica regras de negócio para migração de configurações
    pub fn apply_migration_rules(
        old_config: &Config,
        new_config: &Config,
    ) -> Result<(), CompanySettingsError> {
        // Regra: Migração só pode ser feita em horário de baixa atividade
        // Simulado - em produção seria verificado o horário atual
        let current_hour = 14; // Simulado
        if current_hour >= 9 && current_hour <= 18 {
            return Err(CompanySettingsError::OperationNotAllowed {
                operation: "migration".to_string(),
                reason: "Migração só pode ser feita fora do horário comercial".to_string(),
            });
        }

        // Regra: Configuração deve manter compatibilidade
        if old_config.default_timezone != new_config.default_timezone {
            // Verificar se a mudança é compatível
            if !Self::is_timezone_change_compatible(
                &old_config.default_timezone,
                &new_config.default_timezone,
            ) {
                return Err(CompanySettingsError::ConfigurationInvalid {
                    field: "default_timezone".to_string(),
                    value: new_config.default_timezone.clone(),
                    reason: "Mudança de fuso horário não é compatível com a configuração atual".to_string(),
                });
            }
        }

        Ok(())
    }

    /// Verifica se uma mudança de fuso horário é compatível
    fn is_timezone_change_compatible(old: &str, new: &str) -> bool {
        // Regra: Mudanças entre fusos horários similares são permitidas
        let compatible_groups = [
            vec!["UTC", "GMT"],
            vec!["America/Sao_Paulo", "America/Argentina/Buenos_Aires"],
            vec!["Europe/London", "Europe/Paris", "Europe/Berlin"],
        ];

        for group in &compatible_groups {
            if group.contains(&old) && group.contains(&new) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_creation_rules_success() {
        let result = CompanySettingsBusinessRules::apply_creation_rules(
            "John Doe",
            "john@company.com",
            "America/Sao_Paulo"
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_apply_creation_rules_name_with_spaces_only() {
        let result = CompanySettingsBusinessRules::apply_creation_rules(
            "   ",
            "john@company.com",
            "UTC"
        );
        assert!(matches!(result, Err(CompanySettingsError::ConfigurationInvalid { field, reason, value: _ }) 
            if field == "manager_name" && reason.contains("espaços")));
    }

    #[test]
    fn test_apply_creation_rules_name_starts_with_hyphen() {
        let result = CompanySettingsBusinessRules::apply_creation_rules(
            "-John Doe",
            "john@company.com",
            "UTC"
        );
        assert!(matches!(result, Err(CompanySettingsError::ConfigurationInvalid { field, reason, value: _ }) 
            if field == "manager_name" && reason.contains("hífen")));
    }

    #[test]
    fn test_apply_creation_rules_reserved_email() {
        let result = CompanySettingsBusinessRules::apply_creation_rules(
            "Admin User",
            "admin@system.local",
            "UTC"
        );
        assert!(matches!(result, Err(CompanySettingsError::ConfigurationInvalid { field, reason, value: _ }) 
            if field == "manager_email" && reason.contains("reservado")));
    }

    #[test]
    fn test_apply_creation_rules_generic_email() {
        let result = CompanySettingsBusinessRules::apply_creation_rules(
            "Admin User",
            "admin@company.com",
            "UTC"
        );
        assert!(matches!(result, Err(CompanySettingsError::ConfigurationInvalid { field, reason, value: _ }) 
            if field == "manager_email" && reason.contains("genérico")));
    }

    #[test]
    fn test_apply_creation_rules_extreme_timezone() {
        let result = CompanySettingsBusinessRules::apply_creation_rules(
            "Admin User",
            "admin@specificcompany.com",
            "Asia/Tokyo"
        );
        assert!(matches!(result, Err(CompanySettingsError::ConfigurationInvalid { field, reason, value: _ }) 
            if field == "default_timezone" && reason.contains("muito extremo")));
    }

    #[test]
    fn test_can_remove_config_system_config() {
        let config = Config::new(
            "System Admin".to_string(),
            "admin@system.local".to_string(),
            "UTC".to_string(),
        );
        let result = CompanySettingsBusinessRules::can_remove_config(&config);
        assert!(matches!(result, Err(CompanySettingsError::OperationNotAllowed { operation, reason }) 
            if operation == "remove" && reason.contains("sistema")));
    }

    #[test]
    fn test_is_timezone_change_compatible() {
        assert!(CompanySettingsBusinessRules::is_timezone_change_compatible("UTC", "GMT"));
        assert!(CompanySettingsBusinessRules::is_timezone_change_compatible("Europe/London", "Europe/Paris"));
        assert!(!CompanySettingsBusinessRules::is_timezone_change_compatible("UTC", "America/Sao_Paulo"));
    }
}
