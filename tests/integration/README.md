# Testes Funcionais do TTR CLI

Este diretório contém testes funcionais integrados ao Cargo que executam durante o desenvolvimento e validam o funcionamento completo do CLI TTR.

## 🎯 Objetivos

- **Testes Automatizados**: Executam automaticamente com `cargo test`
- **Validação Completa**: Testam CLI, geração de YAML e HTML
- **Desenvolvimento Ágil**: Feedback imediato durante o desenvolvimento
- **Integração Contínua**: Podem ser executados em CI/CD

## 📁 Estrutura

```
tests/integration/
├── cli_tests.rs      # Testes específicos do CLI
├── e2e_tests.rs      # Testes end-to-end completos
├── test_config.rs    # Configuração e utilitários
└── README.md         # Este arquivo
```

## 🚀 Como Executar

### Executar Todos os Testes
```bash
# Executar todos os testes funcionais
cargo test --test integration

# Com output detalhado
cargo test --test integration -- --nocapture
```

### Executar Testes Específicos
```bash
# Apenas testes de CLI
cargo test --test cli_tests

# Apenas testes E2E
cargo test --test e2e_tests

# Teste específico
cargo test --test cli_tests -- --exact test_cli_help_command
```

### Usar o Script de Teste
```bash
# Executar todos os testes
./test_functional.sh

# Executar testes rápidos
./test_functional.sh quick

# Executar com cobertura
./test_functional.sh coverage

# Executar teste específico
./test_functional.sh test test_cli_help_command
```

## 📋 Tipos de Testes

### 1. Testes de CLI (`cli_tests.rs`)

Testam comandos individuais do CLI:

- ✅ `test_cli_help_command` - Comando de ajuda
- ✅ `test_cli_init_command` - Inicialização do TTR
- ✅ `test_company_creation_and_yaml_validation` - Criação de empresa
- ✅ `test_resource_creation_and_yaml_validation` - Criação de recurso
- ✅ `test_project_creation_and_yaml_validation` - Criação de projeto
- ✅ `test_task_creation_and_yaml_validation` - Criação de tarefa
- ✅ `test_html_generation_and_validation` - Geração de HTML
- ✅ `test_error_handling` - Tratamento de erros
- ✅ `test_list_commands` - Comandos de listagem
- ✅ `test_validation_command` - Comando de validação

### 2. Testes E2E (`e2e_tests.rs`)

Testam fluxos completos:

- ✅ `test_complete_workflow` - Fluxo completo de criação
- ✅ `test_list_commands_workflow` - Fluxo de comandos de listagem
- ✅ `test_validation_workflow` - Fluxo de validação
- ✅ `test_error_handling_workflow` - Fluxo de tratamento de erros
- ✅ `test_html_navigation_workflow` - Fluxo de navegação HTML

## 🔧 Configuração

### Dependências Necessárias

As dependências já estão configuradas no `Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3.20.0"
regex = "1.10"
# ... outras dependências
```

### Configuração de Teste

Os testes usam a classe `TestConfig` para configuração:

```rust
use tests::integration::test_config::TestConfig;

let config = TestConfig::new()
    .with_binary("target/debug/ttr".to_string())
    .with_timeout(30)
    .with_verbose(true);
```

## 📊 Validações Realizadas

### Validação de YAML
- ✅ Sintaxe YAML válida
- ✅ Estrutura esperada (apiVersion, kind, metadata, spec)
- ✅ Conteúdo específico (nomes, códigos, etc.)
- ✅ Chaves obrigatórias presentes

### Validação de HTML
- ✅ Estrutura HTML válida
- ✅ Links funcionais (sem links quebrados)
- ✅ Conteúdo esperado presente
- ✅ Navegação entre páginas

### Validação de CLI
- ✅ Comandos executam com sucesso
- ✅ Saída esperada é gerada
- ✅ Erros são tratados corretamente
- ✅ Argumentos são validados

## 🛠️ Desenvolvimento

### Adicionar Novo Teste

1. **Teste de CLI Simples**:
```rust
#[test]
fn test_meu_novo_comando() {
    let runner = CliTestRunner::new().expect("Failed to create CLI runner");
    
    // Inicializar se necessário
    runner.run_command(&["init", "--name", "Test", "--email", "test@test.com", "--company-name", "Test Co"])
        .expect("Init should work");
    
    // Executar comando
    let output = runner.run_command(&["meu", "comando"])
        .expect("Command should work");
    
    // Validar resultado
    assert!(output.contains("expected output"));
}
```

2. **Teste E2E Completo**:
```rust
#[test]
fn test_meu_fluxo_completo() {
    let runner = E2ETestRunner::new().expect("Failed to create E2E runner");
    
    // Configurar dados
    runner.run_command(&["init", /* ... */]).expect("Init should work");
    // ... outros comandos
    
    // Validar resultado final
    assert!(runner.public_path().exists());
    // ... outras validações
}
```

### Debugging de Testes

```bash
# Executar com output detalhado
cargo test --test cli_tests -- --nocapture

# Executar teste específico com debug
RUST_LOG=debug cargo test --test cli_tests -- --exact test_cli_help_command --nocapture

# Executar apenas um arquivo de teste
cargo test --test cli_tests
```

## 📈 Integração com CI/CD

### GitHub Actions
```yaml
- name: Run Functional Tests
  run: |
    cargo build
    cargo test --test integration -- --nocapture
```

### GitLab CI
```yaml
test_functional:
  script:
    - cargo build
    - cargo test --test integration
```

## 🐛 Troubleshooting

### Problemas Comuns

1. **Binary not found**:
   ```bash
   cargo build
   ```

2. **Test timeout**:
   - Aumentar timeout na configuração
   - Verificar se o sistema não está sobrecarregado

3. **YAML validation fails**:
   - Verificar se o comando CLI está gerando YAML válido
   - Verificar se a estrutura esperada está correta

4. **HTML validation fails**:
   - Verificar se o comando `build` está funcionando
   - Verificar se os templates estão corretos

### Logs de Debug

```bash
# Habilitar logs detalhados
RUST_LOG=debug cargo test --test integration -- --nocapture

# Ver output do stderr
cargo test --test integration -- --nocapture 2>&1 | tee test_output.log
```

## 📚 Recursos Adicionais

- [Documentação do Cargo Test](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [Testes de Integração no Rust](https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests)
- [Tempfile Crate](https://docs.rs/tempfile/latest/tempfile/)
- [Serde YAML](https://docs.rs/serde_yaml/latest/serde_yaml/)
