use crate::application::query::VersioningMiddleware;
use crate::domain::shared::api_versioning::ApiVersion;
use clap::{Args, Subcommand};
use serde_json;

/// Comandos para gerenciar versões da API
#[derive(Debug, Args)]
pub struct ApiVersionArgs {
    #[command(subcommand)]
    pub command: ApiVersionCommand,
}

#[derive(Debug, Subcommand)]
pub enum ApiVersionCommand {
    /// Lista todas as versões disponíveis
    List,
    /// Mostra informações sobre uma versão específica
    Info {
        /// Versão da API (ex: v1.0.0)
        version: String,
    },
    /// Verifica compatibilidade entre duas versões
    Check {
        /// Versão solicitada
        requested: String,
        /// Versão atual
        current: String,
    },
    /// Valida uma query contra uma versão específica
    Validate {
        /// Versão da API para validar
        version: String,
        /// Arquivo JSON com a query
        query_file: String,
    },
    /// Mostra o guia de migração entre versões
    Migrate {
        /// Versão de origem
        from: String,
        /// Versão de destino
        to: String,
    },
}

impl ApiVersionArgs {
    /// Executa o comando de versão da API
    pub fn execute(&self) -> Result<String, String> {
        let middleware = VersioningMiddleware::new();

        match &self.command {
            ApiVersionCommand::List => {
                self.list_versions(&middleware)
            }
            ApiVersionCommand::Info { version } => {
                self.show_version_info(&middleware, version)
            }
            ApiVersionCommand::Check { requested, current } => {
                self.check_compatibility(&middleware, requested, current)
            }
            ApiVersionCommand::Validate { version, query_file } => {
                self.validate_query(&middleware, version, query_file)
            }
            ApiVersionCommand::Migrate { from, to } => {
                self.show_migration_guide(&middleware, from, to)
            }
        }
    }

    /// Lista todas as versões disponíveis
    fn list_versions(&self, middleware: &VersioningMiddleware) -> Result<String, String> {
        let contract_manager = middleware.contract_manager();
        let mut output = String::new();
        
        output.push_str("📋 Versões da API Disponíveis:\n\n");
        
        // Listar versões suportadas
        let versions = vec![
            ApiVersion::new(1, 0, 0),
            ApiVersion::new(1, 1, 0),
        ];

        for version in versions {
            if let Some(contract) = contract_manager.get_contract(&version) {
                output.push_str(&format!("🔹 {} - {} funcionalidades\n", 
                    version, 
                    contract.supported_features.len()
                ));
                
                for feature in &contract.supported_features {
                    output.push_str(&format!("   ✅ {}\n", feature.name));
                }
                
                if !contract.deprecated_features.is_empty() {
                    output.push_str("   ⚠️  Funcionalidades depreciadas:\n");
                    for feature in &contract.deprecated_features {
                        output.push_str(&format!("      - {}\n", feature.name));
                    }
                }
                
                output.push_str("\n");
            }
        }

        output.push_str(&format!("📌 Versão atual: {}\n", middleware.default_version()));
        output.push_str(&format!("📌 Versão mínima suportada: {}\n", 
            contract_manager.minimum_supported()));

        Ok(output)
    }

    /// Mostra informações sobre uma versão específica
    fn show_version_info(&self, middleware: &VersioningMiddleware, version_str: &str) -> Result<String, String> {
        let version: ApiVersion = version_str.parse()
            .map_err(|e| format!("Versão inválida '{}': {}", version_str, e))?;

        let version_info = middleware.get_version_info(&version)
            .ok_or_else(|| format!("Versão '{}' não encontrada", version))?;

        let mut output = String::new();
        output.push_str(&format!("📊 Informações da Versão {}\n\n", version));

        output.push_str("✅ Funcionalidades Suportadas:\n");
        for feature in &version_info.supported_features {
            output.push_str(&format!("   • {}\n", feature));
        }

        if !version_info.deprecated_features.is_empty() {
            output.push_str("\n⚠️  Funcionalidades Depreciadas:\n");
            for feature in &version_info.deprecated_features {
                output.push_str(&format!("   • {}\n", feature));
            }
        }

        if let Some(guide) = &version_info.migration_guide {
            output.push_str(&format!("\n📖 Guia de Migração:\n{}\n", guide));
        }

        Ok(output)
    }

    /// Verifica compatibilidade entre duas versões
    fn check_compatibility(&self, middleware: &VersioningMiddleware, requested: &str, current: &str) -> Result<String, String> {
        let requested_version: ApiVersion = requested.parse()
            .map_err(|e| format!("Versão solicitada inválida '{}': {}", requested, e))?;
        
        let current_version: ApiVersion = current.parse()
            .map_err(|e| format!("Versão atual inválida '{}': {}", current, e))?;

        let compatibility = middleware.check_compatibility(&requested_version, &current_version);

        let mut output = String::new();
        output.push_str(&format!("🔍 Verificação de Compatibilidade\n\n"));
        output.push_str(&format!("📥 Versão Solicitada: {}\n", requested_version));
        output.push_str(&format!("📤 Versão Atual: {}\n\n", current_version));

        if compatibility.is_compatible {
            output.push_str("✅ Compatível\n");
        } else {
            output.push_str("❌ Incompatível\n");
        }

        if !compatibility.warnings.is_empty() {
            output.push_str("\n⚠️  Avisos:\n");
            for warning in &compatibility.warnings {
                output.push_str(&format!("   • {}\n", warning));
            }
        }

        if !compatibility.breaking_changes.is_empty() {
            output.push_str("\n💥 Mudanças que Quebram Compatibilidade:\n");
            for change in &compatibility.breaking_changes {
                output.push_str(&format!("   • {}\n", change));
            }
        }

        if !compatibility.new_features.is_empty() {
            output.push_str("\n🆕 Novas Funcionalidades:\n");
            for feature in &compatibility.new_features {
                output.push_str(&format!("   • {}\n", feature));
            }
        }

        Ok(output)
    }

    /// Valida uma query contra uma versão específica
    fn validate_query(&self, middleware: &VersioningMiddleware, version_str: &str, query_file: &str) -> Result<String, String> {
        let version: ApiVersion = version_str.parse()
            .map_err(|e| format!("Versão inválida '{}': {}", version_str, e))?;

        // Ler arquivo de query
        let query_content = std::fs::read_to_string(query_file)
            .map_err(|e| format!("Erro ao ler arquivo '{}': {}", query_file, e))?;

        // Parse da query JSON
        let query: crate::domain::shared::query_parser::Query = serde_json::from_str(&query_content)
            .map_err(|e| format!("Erro ao fazer parse da query JSON: {}", e))?;

        // Validar query
        let validation_result = middleware.contract_manager().validate_query(&query, &version);

        let mut output = String::new();
        output.push_str(&format!("🔍 Validação da Query contra {}\n\n", version));

        if validation_result.is_success() {
            output.push_str("✅ Query válida para esta versão\n");
            
            let warnings = validation_result.warnings();
            if !warnings.is_empty() {
                output.push_str("\n⚠️  Avisos:\n");
                for warning in warnings {
                    output.push_str(&format!("   • {}\n", warning));
                }
            }
        } else {
            output.push_str("❌ Query inválida para esta versão\n");
            if let Some(error) = validation_result.error_message() {
                output.push_str(&format!("   Erro: {}\n", error));
            }
        }

        Ok(output)
    }

    /// Mostra o guia de migração entre versões
    fn show_migration_guide(&self, middleware: &VersioningMiddleware, from: &str, to: &str) -> Result<String, String> {
        let from_version: ApiVersion = from.parse()
            .map_err(|e| format!("Versão de origem inválida '{}': {}", from, e))?;
        
        let to_version: ApiVersion = to.parse()
            .map_err(|e| format!("Versão de destino inválida '{}': {}", to, e))?;

        let compatibility = middleware.check_compatibility(&from_version, &to_version);
        let changes = middleware.contract_manager().get_version_changes(&from_version, &to_version);

        let mut output = String::new();
        output.push_str(&format!("📖 Guia de Migração: {} → {}\n\n", from_version, to_version));

        if !changes.is_empty() {
            output.push_str("🔄 Mudanças de Versão:\n");
            for change in changes {
                output.push_str(&format!("   • {}\n", change));
            }
            output.push_str("\n");
        }

        if !compatibility.warnings.is_empty() {
            output.push_str("⚠️  Avisos de Compatibilidade:\n");
            for warning in &compatibility.warnings {
                output.push_str(&format!("   • {}\n", warning));
            }
            output.push_str("\n");
        }

        if !compatibility.breaking_changes.is_empty() {
            output.push_str("💥 Mudanças que Quebram Compatibilidade:\n");
            for change in &compatibility.breaking_changes {
                output.push_str(&format!("   • {}\n", change));
            }
            output.push_str("\n");
        }

        if !compatibility.new_features.is_empty() {
            output.push_str("🆕 Novas Funcionalidades Disponíveis:\n");
            for feature in &compatibility.new_features {
                output.push_str(&format!("   • {}\n", feature));
            }
            output.push_str("\n");
        }

        // Adicionar guia específico se disponível
        if let Some(contract) = middleware.contract_manager().get_contract(&to_version) {
            if let Some(guide) = &contract.migration_guide {
                output.push_str(&format!("📋 Guia de Migração Específico:\n{}\n", guide));
            }
        }

        Ok(output)
    }
}
