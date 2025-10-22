use serde::{Deserialize, Serialize};
use std::fmt;

/// Versão da API de queries
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct ApiVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ApiVersion {
    /// Cria uma nova versão da API
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    /// Versão atual da API
    pub fn current() -> Self {
        Self::new(1, 0, 0)
    }

    /// Versão mais antiga suportada
    pub fn minimum_supported() -> Self {
        Self::new(1, 0, 0)
    }

    /// Verifica se esta versão é compatível com outra
    pub fn is_compatible_with(&self, other: &ApiVersion) -> bool {
        // Versões são compatíveis se têm o mesmo major version
        self.major == other.major
    }

    /// Verifica se esta versão é mais recente que outra
    pub fn is_newer_than(&self, other: &ApiVersion) -> bool {
        self > other
    }

    /// Verifica se esta versão é mais antiga que outra
    pub fn is_older_than(&self, other: &ApiVersion) -> bool {
        self < other
    }
}

impl fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl std::str::FromStr for ApiVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_start_matches('v');
        let parts: Vec<&str> = s.split('.').collect();

        if parts.len() != 3 {
            return Err(format!("Invalid version format: {}", s));
        }

        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| format!("Invalid major version: {}", parts[0]))?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| format!("Invalid minor version: {}", parts[1]))?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|_| format!("Invalid patch version: {}", parts[2]))?;

        Ok(ApiVersion::new(major, minor, patch))
    }
}

/// Informações de compatibilidade entre versões
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    pub is_compatible: bool,
    pub warnings: Vec<String>,
    pub breaking_changes: Vec<String>,
    pub new_features: Vec<String>,
}

impl CompatibilityInfo {
    /// Cria informações de compatibilidade
    pub fn new(
        is_compatible: bool,
        warnings: Vec<String>,
        breaking_changes: Vec<String>,
        new_features: Vec<String>,
    ) -> Self {
        Self {
            is_compatible,
            warnings,
            breaking_changes,
            new_features,
        }
    }

    /// Cria informações de compatibilidade total
    pub fn fully_compatible() -> Self {
        Self::new(true, Vec::new(), Vec::new(), Vec::new())
    }

    /// Cria informações de incompatibilidade
    pub fn incompatible(reason: String) -> Self {
        Self::new(false, Vec::new(), vec![reason], Vec::new())
    }
}

/// Gerenciador de versionamento da API
pub struct ApiVersionManager {
    current_version: ApiVersion,
    minimum_supported: ApiVersion,
}

impl ApiVersionManager {
    /// Cria um novo gerenciador de versões
    pub fn new() -> Self {
        Self {
            current_version: ApiVersion::current(),
            minimum_supported: ApiVersion::minimum_supported(),
        }
    }

    /// Obtém a versão atual da API
    pub fn current_version(&self) -> &ApiVersion {
        &self.current_version
    }

    /// Obtém a versão mínima suportada
    pub fn minimum_supported(&self) -> &ApiVersion {
        &self.minimum_supported
    }

    /// Verifica compatibilidade entre duas versões
    pub fn check_compatibility(&self, requested: &ApiVersion, actual: &ApiVersion) -> CompatibilityInfo {
        if !requested.is_compatible_with(actual) {
            return CompatibilityInfo::incompatible(format!(
                "Major version mismatch: requested {}, actual {}",
                requested, actual
            ));
        }

        if requested.is_older_than(&self.minimum_supported) {
            return CompatibilityInfo::incompatible(format!(
                "Version {} is no longer supported. Minimum supported: {}",
                requested, self.minimum_supported
            ));
        }

        let mut warnings = Vec::new();
        let mut new_features = Vec::new();

        if requested.is_older_than(actual) {
            warnings.push(format!(
                "Using older version {} than current {}. Some features may not be available.",
                requested, actual
            ));
        } else if requested.is_newer_than(actual) {
            new_features.push(format!(
                "Using newer version {} than current {}. New features available.",
                requested, actual
            ));
        }

        CompatibilityInfo::new(true, warnings, Vec::new(), new_features)
    }

    /// Valida se uma versão é suportada
    pub fn is_version_supported(&self, version: &ApiVersion) -> bool {
        version >= &self.minimum_supported && version.major == self.current_version.major
    }

    /// Obtém informações sobre mudanças entre versões
    pub fn get_version_changes(&self, from: &ApiVersion, to: &ApiVersion) -> Vec<String> {
        let mut changes = Vec::new();

        if from.major != to.major {
            changes.push(format!("Major version change: {} -> {}", from, to));
        }

        if from.minor != to.minor {
            changes.push(format!("Minor version change: {} -> {}", from, to));
        }

        if from.patch != to.patch {
            changes.push(format!("Patch version change: {} -> {}", from, to));
        }

        changes
    }
}

impl Default for ApiVersionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_version_creation() {
        let version = ApiVersion::new(1, 2, 3);
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_api_version_display() {
        let version = ApiVersion::new(1, 2, 3);
        assert_eq!(format!("{}", version), "v1.2.3");
    }

    #[test]
    fn test_api_version_parsing() {
        let version: ApiVersion = "v1.2.3".parse().unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_api_version_comparison() {
        let v1 = ApiVersion::new(1, 0, 0);
        let v2 = ApiVersion::new(1, 1, 0);
        let v3 = ApiVersion::new(2, 0, 0);

        assert!(v2 > v1);
        assert!(v3 > v2);
        assert!(v1.is_compatible_with(&v2));
        assert!(!v1.is_compatible_with(&v3));
    }

    #[test]
    fn test_compatibility_info() {
        let info = CompatibilityInfo::fully_compatible();
        assert!(info.is_compatible);
        assert!(info.warnings.is_empty());
        assert!(info.breaking_changes.is_empty());
        assert!(info.new_features.is_empty());
    }

    #[test]
    fn test_api_version_manager() {
        let manager = ApiVersionManager::new();
        let current = manager.current_version();
        let minimum = manager.minimum_supported();

        assert_eq!(current.major, 1);
        assert_eq!(minimum.major, 1);
        assert!(manager.is_version_supported(current));
    }

    #[test]
    fn test_compatibility_check() {
        let manager = ApiVersionManager::new();
        let v1_0_0 = ApiVersion::new(1, 0, 0);
        let v1_1_0 = ApiVersion::new(1, 1, 0);
        let v2_0_0 = ApiVersion::new(2, 0, 0);

        let compat1 = manager.check_compatibility(&v1_0_0, &v1_1_0);
        assert!(compat1.is_compatible);

        let compat2 = manager.check_compatibility(&v1_0_0, &v2_0_0);
        assert!(!compat2.is_compatible);
    }
}
