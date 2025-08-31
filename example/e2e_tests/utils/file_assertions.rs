use std::path::{Path, PathBuf};
use std::fs;
use serde_yaml;

/// Utilitários para validar arquivos e diretórios nos testes
pub struct FileAssertions;

impl FileAssertions {
    /// Verifica se um diretório existe
    pub fn assert_directory_exists(path: &Path) -> Result<(), String> {
        if path.exists() && path.is_dir() {
            Ok(())
        } else {
            Err(format!("Directory does not exist: {:?}", path))
        }
    }
    
    /// Verifica se um arquivo existe
    pub fn assert_file_exists(path: &Path) -> Result<(), String> {
        if path.exists() && path.is_file() {
            Ok(())
        } else {
            Err(format!("File does not exist: {:?}", path))
        }
    }
    
    /// Verifica se um arquivo não existe
    pub fn assert_file_not_exists(path: &Path) -> Result<(), String> {
        if !path.exists() {
            Ok(())
        } else {
            Err(format!("File should not exist: {:?}", path))
        }
    }
    
    /// Verifica se um arquivo tem conteúdo
    pub fn assert_file_not_empty(path: &Path) -> Result<(), String> {
        Self::assert_file_exists(path)?;
        
        let metadata = fs::metadata(path)
            .map_err(|e| format!("Failed to read file metadata: {}", e))?;
        
        if metadata.len() > 0 {
            Ok(())
        } else {
            Err(format!("File is empty: {:?}", path))
        }
    }
    
    /// Verifica se um arquivo contém texto específico
    pub fn assert_file_contains(path: &Path, expected_text: &str) -> Result<(), String> {
        Self::assert_file_exists(path)?;
        
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        if content.contains(expected_text) {
            Ok(())
        } else {
            Err(format!("File does not contain '{}': {:?}", expected_text, path))
        }
    }
    
    /// Verifica se um arquivo YAML é válido
    pub fn assert_valid_yaml(path: &Path) -> Result<(), String> {
        Self::assert_file_exists(path)?;
        
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        serde_yaml::from_str::<serde_yaml::Value>(&content)
            .map_err(|e| format!("Invalid YAML in file {:?}: {}", path, e))?;
        
        Ok(())
    }
    
    /// Verifica se um arquivo YAML contém chave específica
    pub fn assert_yaml_contains_key(path: &Path, key: &str) -> Result<(), String> {
        Self::assert_valid_yaml(path)?;
        
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse YAML: {}", e))?;
        
        if let Some(value) = yaml.get(key) {
            if !value.is_null() {
                Ok(())
            } else {
                Err(format!("Key '{}' exists but is null in file {:?}", key, path))
            }
        } else {
            Err(format!("Key '{}' not found in file {:?}", key, path))
        }
    }
    
    /// Verifica se um arquivo YAML contém valor específico para uma chave
    pub fn assert_yaml_contains_value(path: &Path, key: &str, expected_value: &str) -> Result<(), String> {
        Self::assert_valid_yaml(path)?;
        
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse YAML: {}", e))?;
        
        if let Some(value) = yaml.get(key) {
            if let Some(str_value) = value.as_str() {
                if str_value == expected_value {
                    Ok(())
                } else {
                    Err(format!("Expected '{}' for key '{}', got '{}' in file {:?}", 
                               expected_value, key, str_value, path))
                }
            } else {
                Err(format!("Key '{}' is not a string in file {:?}", key, path))
            }
        } else {
            Err(format!("Key '{}' not found in file {:?}", key, path))
        }
    }
    
    /// Verifica se um arquivo CSV é válido
    pub fn assert_valid_csv(path: &Path) -> Result<(), String> {
        Self::assert_file_exists(path)?;
        
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Err("CSV file is empty".to_string());
        }
        
        // Verificar se tem pelo menos cabeçalho
        let header = lines[0];
        if header.trim().is_empty() {
            return Err("CSV header is empty".to_string());
        }
        
        // Verificar se tem pelo menos uma linha de dados
        if lines.len() < 2 {
            return Err("CSV file has no data rows".to_string());
        }
        
        Ok(())
    }
    
    /// Verifica se um arquivo HTML é válido
    pub fn assert_valid_html(path: &Path) -> Result<(), String> {
        Self::assert_file_exists(path)?;
        
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        
        // Verificações básicas de HTML
        if !content.contains("<html") && !content.contains("<!DOCTYPE") {
            return Err("File does not appear to be valid HTML".to_string());
        }
        
        if content.contains("<body") || content.contains("<head") {
            Ok(())
        } else {
            Err("HTML file missing basic structure".to_string())
        }
    }
    
    /// Verifica se um arquivo tem extensão específica
    pub fn assert_file_extension(path: &Path, expected_extension: &str) -> Result<(), String> {
        if let Some(extension) = path.extension() {
            if extension == expected_extension {
                Ok(())
            } else {
                Err(format!("Expected extension '{}', got '{:?}' for file {:?}", 
                           expected_extension, extension, path))
            }
        } else {
            Err(format!("File has no extension: {:?}", path))
        }
    }
    
    /// Verifica se um diretório contém arquivos com extensão específica
    pub fn assert_directory_contains_files_with_extension(dir_path: &Path, extension: &str) -> Result<(), String> {
        Self::assert_directory_exists(dir_path)?;
        
        let entries = fs::read_dir(dir_path)
            .map_err(|e| format!("Failed to read directory: {}", e))?;
        
        let mut found_files = 0;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(file_extension) = path.extension() {
                    if file_extension == extension {
                        found_files += 1;
                    }
                }
            }
        }
        
        if found_files > 0 {
            Ok(())
        } else {
            Err(format!("No files with extension '{}' found in directory {:?}", extension, dir_path))
        }
    }
    
    /// Verifica se um diretório tem estrutura específica
    pub fn assert_directory_structure(dir_path: &Path, expected_structure: &[&str]) -> Result<(), String> {
        Self::assert_directory_exists(dir_path)?;
        
        for expected_item in expected_structure {
            let item_path = dir_path.join(expected_item);
            if !item_path.exists() {
                return Err(format!("Expected item '{}' not found in directory {:?}", expected_item, dir_path));
            }
        }
        
        Ok(())
    }
    
    /// Verifica se um arquivo tem tamanho mínimo
    pub fn assert_file_min_size(path: &Path, min_size_bytes: u64) -> Result<(), String> {
        Self::assert_file_exists(path)?;
        
        let metadata = fs::metadata(path)
            .map_err(|e| format!("Failed to read file metadata: {}", e))?;
        
        if metadata.len() >= min_size_bytes {
            Ok(())
        } else {
            Err(format!("File size {} bytes is less than minimum {} bytes: {:?}", 
                       metadata.len(), min_size_bytes, path))
        }
    }
    
    /// Verifica se um arquivo foi modificado recentemente
    pub fn assert_file_recently_modified(path: &Path, max_age_seconds: u64) -> Result<(), String> {
        Self::assert_file_exists(path)?;
        
        let metadata = fs::metadata(path)
            .map_err(|e| format!("Failed to read file metadata: {}", e))?;
        
        let modified_time = metadata.modified()
            .map_err(|e| format!("Failed to get modification time: {}", e))?;
        
        let now = std::time::SystemTime::now();
        let age = now.duration_since(modified_time)
            .map_err(|e| format!("Failed to calculate file age: {}", e))?;
        
        if age.as_secs() <= max_age_seconds {
            Ok(())
        } else {
            Err(format!("File is too old ({} seconds): {:?}", age.as_secs(), path))
        }
    }
}

/// Builder para criar asserções complexas de arquivos
pub struct FileAssertionBuilder {
    path: PathBuf,
    checks: Vec<Box<dyn Fn(&Path) -> Result<(), String>>>,
}

impl FileAssertionBuilder {
    /// Cria um novo builder para um arquivo
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            checks: Vec::new(),
        }
    }
    
    /// Adiciona verificação de existência
    pub fn exists(mut self) -> Self {
        self.checks.push(Box::new(|p| FileAssertions::assert_file_exists(p)));
        self
    }
    
    /// Adiciona verificação de não existência
    pub fn not_exists(mut self) -> Self {
        self.checks.push(Box::new(|p| FileAssertions::assert_file_not_exists(p)));
        self
    }
    
    /// Adiciona verificação de conteúdo
    pub fn contains(mut self, text: &'static str) -> Self {
        let text = text.to_string();
        self.checks.push(Box::new(move |p| FileAssertions::assert_file_contains(p, &text)));
        self
    }
    
    /// Adiciona verificação de extensão
    pub fn has_extension(mut self, extension: &'static str) -> Self {
        let extension = extension.to_string();
        self.checks.push(Box::new(move |p| FileAssertions::assert_file_extension(p, &extension)));
        self
    }
    
    /// Adiciona verificação de tamanho mínimo
    pub fn min_size(mut self, min_size: u64) -> Self {
        self.checks.push(Box::new(move |p| FileAssertions::assert_file_min_size(p, min_size)));
        self
    }
    
    /// Adiciona verificação de modificação recente
    pub fn recently_modified(mut self, max_age_seconds: u64) -> Self {
        self.checks.push(Box::new(move |p| FileAssertions::assert_file_recently_modified(p, max_age_seconds)));
        self
    }
    
    /// Adiciona verificação de YAML válido
    pub fn valid_yaml(mut self) -> Self {
        self.checks.push(Box::new(|p| FileAssertions::assert_valid_yaml(p)));
        self
    }
    
    /// Adiciona verificação de CSV válido
    pub fn valid_csv(mut self) -> Self {
        self.checks.push(Box::new(|p| FileAssertions::assert_valid_csv(p)));
        self
    }
    
    /// Adiciona verificação de HTML válido
    pub fn valid_html(mut self) -> Self {
        self.checks.push(Box::new(|p| FileAssertions::assert_valid_html(p)));
        self
    }
    
    /// Executa todas as verificações
    pub fn assert(self) -> Result<(), String> {
        for check in self.checks {
            check(&self.path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;
    
    #[test]
    fn test_file_assertions() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        // Criar arquivo de teste
        fs::write(&test_file, "Hello, World!").unwrap();
        
        // Testar asserções
        assert!(FileAssertions::assert_file_exists(&test_file).is_ok());
        assert!(FileAssertions::assert_file_not_empty(&test_file).is_ok());
        assert!(FileAssertions::assert_file_contains(&test_file, "Hello").is_ok());
        assert!(FileAssertions::assert_file_extension(&test_file, "txt").is_ok());
    }
    
    #[test]
    fn test_file_assertion_builder() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.yaml");
        
        // Criar arquivo YAML de teste
        fs::write(&test_file, "name: Test\nvalue: 42").unwrap();
        
        // Usar builder para asserções
        let result = FileAssertionBuilder::new(test_file)
            .exists()
            .valid_yaml()
            .contains("Test")
            .has_extension("yaml")
            .assert();
        
        assert!(result.is_ok());
    }
}
