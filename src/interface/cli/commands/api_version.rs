use crate::application::query::VersioningMiddleware;
use crate::domain::shared::api_versioning::ApiVersion;
use clap::{Args, Subcommand};
use serde_json;

/// Commands for managing API versions
#[derive(Debug, Args)]
pub struct ApiVersionArgs {
    #[command(subcommand)]
    pub command: ApiVersionCommand,
}

#[derive(Debug, Subcommand)]
pub enum ApiVersionCommand {
    /// List all available versions
    List,
    /// Show information about a specific version
    Info {
        /// API version (e.g., v1.0.0)
        version: String,
    },
    /// Check compatibility between two versions
    Check {
        /// Requested version
        requested: String,
        /// Current version
        current: String,
    },
    /// Validate a query against a specific version
    Validate {
        /// API version to validate against
        version: String,
        /// JSON file with the query
        query_file: String,
    },
    /// Show migration guide between versions
    Migrate {
        /// Source version
        from: String,
        /// Target version
        to: String,
    },
}

impl ApiVersionArgs {
    /// Execute the API version command
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

    /// List all available versions
    fn list_versions(&self, middleware: &VersioningMiddleware) -> Result<String, String> {
        let contract_manager = middleware.contract_manager();
        let mut output = String::new();
        
        output.push_str("📋 Available API Versions:\n\n");
        
        // List supported versions
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

        output.push_str(&format!("📌 Current version: {}\n", middleware.default_version()));
        output.push_str(&format!("📌 Minimum supported version: {}\n", 
            contract_manager.minimum_supported()));

        Ok(output)
    }

    /// Show information about a specific version
    fn show_version_info(&self, middleware: &VersioningMiddleware, version_str: &str) -> Result<String, String> {
        let version: ApiVersion = version_str.parse()
            .map_err(|e| format!("Invalid version '{}': {}", version_str, e))?;

        let version_info = middleware.get_version_info(&version)
            .ok_or_else(|| format!("Version '{}' not found", version))?;

        let mut output = String::new();
        output.push_str(&format!("📊 Version Information {}\n\n", version));

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
            output.push_str(&format!("\n📖 Migration Guide:\n{}\n", guide));
        }

        Ok(output)
    }

    /// Check compatibility between two versions
    fn check_compatibility(&self, middleware: &VersioningMiddleware, requested: &str, current: &str) -> Result<String, String> {
        let requested_version: ApiVersion = requested.parse()
            .map_err(|e| format!("Invalid requested version '{}': {}", requested, e))?;
        
        let current_version: ApiVersion = current.parse()
            .map_err(|e| format!("Invalid current version '{}': {}", current, e))?;

        let compatibility = middleware.check_compatibility(&requested_version, &current_version);

        let mut output = String::new();
        output.push_str(&format!("🔍 Compatibility Check\n\n"));
        output.push_str(&format!("📥 Requested Version: {}\n", requested_version));
        output.push_str(&format!("📤 Current Version: {}\n\n", current_version));

        if compatibility.is_compatible {
            output.push_str("✅ Compatible\n");
        } else {
            output.push_str("❌ Incompatible\n");
        }

        if !compatibility.warnings.is_empty() {
            output.push_str("\n⚠️  Avisos:\n");
            for warning in &compatibility.warnings {
                output.push_str(&format!("   • {}\n", warning));
            }
        }

        if !compatibility.breaking_changes.is_empty() {
            output.push_str("\n💥 Breaking Changes:\n");
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

    /// Validate a query against a specific version
    fn validate_query(&self, middleware: &VersioningMiddleware, version_str: &str, query_file: &str) -> Result<String, String> {
        let version: ApiVersion = version_str.parse()
            .map_err(|e| format!("Invalid version '{}': {}", version_str, e))?;

        // Ler arquivo de query
        let query_content = std::fs::read_to_string(query_file)
            .map_err(|e| format!("Erro ao ler arquivo '{}': {}", query_file, e))?;

        // Parse da query JSON
        let query: crate::domain::shared::query_parser::Query = serde_json::from_str(&query_content)
            .map_err(|e| format!("Erro ao fazer parse da query JSON: {}", e))?;

        // Validar query
        let validation_result = middleware.contract_manager().validate_query(&query, &version);

        let mut output = String::new();
        output.push_str(&format!("🔍 Query Validation against {}\n\n", version));

        if validation_result.is_success() {
            output.push_str("✅ Query valid for this version\n");
            
            let warnings = validation_result.warnings();
            if !warnings.is_empty() {
                output.push_str("\n⚠️  Avisos:\n");
                for warning in warnings {
                    output.push_str(&format!("   • {}\n", warning));
                }
            }
        } else {
            output.push_str("❌ Query invalid for this version\n");
            if let Some(error) = validation_result.error_message() {
                output.push_str(&format!("   Erro: {}\n", error));
            }
        }

        Ok(output)
    }

    /// Show migration guide between versions
    fn show_migration_guide(&self, middleware: &VersioningMiddleware, from: &str, to: &str) -> Result<String, String> {
        let from_version: ApiVersion = from.parse()
            .map_err(|e| format!("Invalid source version '{}': {}", from, e))?;
        
        let to_version: ApiVersion = to.parse()
            .map_err(|e| format!("Invalid target version '{}': {}", to, e))?;

        let compatibility = middleware.check_compatibility(&from_version, &to_version);
        let changes = middleware.contract_manager().get_version_changes(&from_version, &to_version);

        let mut output = String::new();
        output.push_str(&format!("📖 Migration Guide: {} → {}\n\n", from_version, to_version));

        if !changes.is_empty() {
            output.push_str("🔄 Version Changes:\n");
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
            output.push_str("💥 Breaking Changes:\n");
            for change in &compatibility.breaking_changes {
                output.push_str(&format!("   • {}\n", change));
            }
            output.push_str("\n");
        }

        if !compatibility.new_features.is_empty() {
            output.push_str("🆕 New Features Available:\n");
            for feature in &compatibility.new_features {
                output.push_str(&format!("   • {}\n", feature));
            }
            output.push_str("\n");
        }

        // Add specific guide if available
        if let Some(contract) = middleware.contract_manager().get_contract(&to_version) {
            if let Some(guide) = &contract.migration_guide {
                output.push_str(&format!("📋 Specific Migration Guide:\n{}\n", guide));
            }
        }

        Ok(output)
    }
}
