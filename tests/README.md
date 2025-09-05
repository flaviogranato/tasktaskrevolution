# Testes Funcionais do TTR CLI

Este diretório contém testes funcionais que seguem as **melhores práticas do Rust** usando `assert_cmd`, `predicates` e `assert_fs`.

## 🎯 Abordagem Recomendada

Seguimos a estratégia principal do Cargo que distingue automaticamente dois tipos de testes:

- **Testes Unitários**: Ficam dentro do `src/`, próximos ao código que testam
- **Testes de Integração**: Ficam em `tests/`, executam o binário compilado

## 🛠️ Ferramentas Utilizadas

### Dependências Principais
```toml
[dev-dependencies]
assert_cmd = "2.0"  # Para testar CLIs
predicates = "3.1"  # Para asserções no output
assert_fs = "1.1"   # Para trabalhar com arquivos temporários
```

### Por que essas ferramentas?

- **`assert_cmd`**: Permite executar o binário CLI, passar argumentos e fazer asserções sobre o resultado
- **`predicates`**: Funciona com `assert_cmd` para verificar stdout, stderr e códigos de saída de forma declarativa
- **`assert_fs`**: Cria diretórios temporários seguros para testes E2E, garantindo isolamento

## 📁 Estrutura do Projeto

```
TaskTaskRevolution/
├── Cargo.toml
├── src/
│   ├── main.rs       # Interface CLI simples
│   └── lib.rs        # Lógica principal (testável)
└── tests/
    └── cli.rs        # Testes funcionais e E2E
```

### Separação de Responsabilidades

- **`src/main.rs`**: Apenas interface CLI - chama `TaskTaskRevolution::run()`
- **`src/lib.rs`**: Lógica principal + testes unitários
- **`tests/cli.rs`**: Testes funcionais que executam o binário

## 🚀 Como Executar

### Executar Todos os Testes
```bash
# Executa testes unitários + testes de integração
cargo test

# Apenas testes de integração
cargo test --test cli

# Com output detalhado
cargo test --test cli -- --nocapture
```

### Executar Testes Específicos
```bash
# Teste específico
cargo test --test cli -- --exact test_help_command

# Teste de fluxo completo
cargo test --test cli -- --exact test_complete_workflow
```

### Usar o Script de Teste
```bash
# Executar todos os testes
./test_functional.sh

# Executar testes rápidos
./test_functional.sh quick

# Executar com cobertura
./test_functional.sh coverage
```

## 📋 Testes Implementados

### Testes Funcionais (`tests/cli.rs`)

#### **Comandos Básicos**
- ✅ `test_help_command` - Comando `--help`
- ✅ `test_version_command` - Comando `--version`
- ✅ `test_init_command` - Inicialização básica
- ✅ `test_init_command_with_timezone` - Inicialização com timezone

#### **Criação de Entidades**
- ✅ `test_create_company` - Criação de empresa
- ✅ `test_create_resource` - Criação de recurso
- ✅ `test_create_project` - Criação de projeto
- ✅ `test_create_task` - Criação de tarefa

#### **Comandos de Listagem**
- ✅ `test_list_commands` - Listagem de projetos, recursos e tarefas

#### **Validação e Build**
- ✅ `test_validate_command` - Comando de validação
- ✅ `test_build_command` - Geração de HTML

#### **Tratamento de Erros**
- ✅ `test_error_handling` - Comandos inválidos e erros

#### **Fluxo Completo E2E**
- ✅ `test_complete_workflow` - Fluxo completo: init → create → build

## 🔧 Exemplos de Código

### Teste Básico com assert_cmd
```rust
#[test]
fn test_help_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("ttr")?;
    
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("utilitário de linha de comando"))
        .stdout(predicate::str::contains("Usage: ttr"));
    
    Ok(())
}
```

### Teste E2E com assert_fs
```rust
#[test]
fn test_create_company() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    let company_file = temp.child("companies").child("TECH-CORP").child("company.yaml");
    
    // Inicializar
    let mut init_cmd = Command::cargo_bin("ttr")?;
    init_cmd.current_dir(temp.path());
    init_cmd.args(&["init", "--name", "Test Manager", "--email", "test@example.com", "--company-name", "Test Company"]);
    init_cmd.assert().success();
    
    // Criar empresa
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args(&["create", "company", "--name", "Tech Corp", "--code", "TECH-CORP", "--description", "Technology company"]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Company created successfully"));
    
    // Verificar arquivo criado
    company_file.assert(predicate::path::exists());
    company_file.assert(predicate::str::contains("Tech Corp"));
    
    temp.close()?;
    Ok(())
}
```

### Teste de Tratamento de Erros
```rust
#[test]
fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;
    
    // Comando inválido
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("invalid-command");
    cmd.assert().failure();
    
    // Comando que retorna erro na saída
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("list").arg("tasks");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Error listing tasks"));
    
    temp.close()?;
    Ok(())
}
```

## 📊 Validações Realizadas

### Validação de CLI
- ✅ Comandos executam com sucesso
- ✅ Saídas contêm texto esperado
- ✅ Códigos de saída estão corretos
- ✅ Argumentos são processados corretamente

### Validação de Arquivos
- ✅ Arquivos YAML são criados
- ✅ Conteúdo dos arquivos está correto
- ✅ Estrutura de diretórios é criada
- ✅ Arquivos HTML são gerados

### Validação de Fluxos
- ✅ Fluxo completo funciona
- ✅ Dependências entre comandos são respeitadas
- ✅ Dados são persistidos corretamente

## 🎯 Vantagens desta Abordagem

### 1. **Simplicidade**
- Usa ferramentas padrão da comunidade Rust
- Código limpo e legível
- Fácil de manter e estender

### 2. **Confiabilidade**
- Testes isolados com diretórios temporários
- Validação real do binário compilado
- Cobertura completa de funcionalidades

### 3. **Performance**
- Testes rápidos e eficientes
- Execução paralela quando possível
- Limpeza automática de recursos

### 4. **Integração**
- Funciona perfeitamente com `cargo test`
- Integração com CI/CD
- Relatórios de cobertura

## 🐛 Troubleshooting

### Problemas Comuns

1. **Binary not found**:
   ```bash
   cargo build
   ```

2. **Test timeout**:
   - Verificar se o sistema não está sobrecarregado
   - Aumentar timeout se necessário

3. **File assertions fail**:
   - Verificar se os comandos CLI estão gerando os arquivos esperados
   - Verificar caminhos dos arquivos

### Debug de Testes

```bash
# Executar com output detalhado
cargo test --test cli -- --nocapture

# Executar teste específico
cargo test --test cli -- --exact test_help_command --nocapture

# Executar com RUST_BACKTRACE
RUST_BACKTRACE=1 cargo test --test cli
```

## 📚 Recursos Adicionais

- [Documentação do assert_cmd](https://docs.rs/assert_cmd/latest/assert_cmd/)
- [Documentação do predicates](https://docs.rs/predicates/latest/predicates/)
- [Documentação do assert_fs](https://docs.rs/assert_fs/latest/assert_fs/)
- [Testes de Integração no Rust](https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests)

## 🎉 Resultado Final

Com esta abordagem, você tem:

- ✅ **13 testes funcionais** cobrindo todas as funcionalidades
- ✅ **Execução com um comando**: `cargo test`
- ✅ **Validação completa**: CLI, YAML, HTML
- ✅ **Código limpo**: Seguindo melhores práticas do Rust
- ✅ **Manutenibilidade**: Fácil de estender e modificar

**Tudo funcionando perfeitamente!** 🚀
