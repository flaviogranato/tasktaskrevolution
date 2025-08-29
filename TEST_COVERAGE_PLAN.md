# Plano de Cobertura de Testes - 85% Target

## 📊 Status Atual da Cobertura

**Cobertura Atual**: 44.53% (1474/3310 linhas)
**Meta**: 85% (2814/3310 linhas)
**Gap**: +1340 linhas a serem cobertas

## 🎯 Estratégia Geral

### Fase 1: Testes Unitários Críticos (Prioridade ALTA)
**Objetivo**: Cobrir 60% das linhas não testadas
**Estimativa**: 3-4 dias
**Foco**: Domínio e aplicação

### Fase 2: Testes de Integração (Prioridade ALTA)
**Objetivo**: Cobrir 25% das linhas não testadas
**Estimativa**: 2-3 dias
**Foco**: Persistência e repositórios

### Fase 3: Testes CLI e End-to-End (Prioridade MÉDIA)
**Objetivo**: Cobrir 15% das linhas não testadas
**Estimativa**: 2-3 dias
**Foco**: Interface e fluxos completos

## 📋 Análise Detalhada por Módulo

### 🚨 Módulos com 0% de Cobertura (CRÍTICO)

#### 1. `src/interface/cli.rs` (0/406 linhas)
- **Problema**: CLI não testado
- **Solução**: Testes unitários + testes de integração CLI
- **Prioridade**: ALTA

#### 2. `src/domain/shared/` (0% em vários arquivos)
- **`errors.rs`**: 10/96 linhas (10.4%)
- **`command.rs`**: 0/40 linhas
- **`observer.rs`**: 0/81 linhas
- **`repository.rs`**: 0/104 linhas
- **`specification.rs`**: 0/100 linhas
- **`factory.rs`**: 0/18 linhas
- **`validatable.rs`**: 0/28 linhas
- **`convertable.rs`**: 0/8 linhas

#### 3. `src/domain/company_settings/errors.rs` (0/37 linhas)
#### 4. `src/domain/project_management/errors.rs` (0/38 linhas)
#### 5. `src/domain/resource_management/errors.rs` (0/51 linhas)
#### 6. `src/domain/task_management/errors.rs` (0/62 linhas)
#### 7. `src/infrastructure/errors.rs` (0/69 linhas)

### ⚠️ Módulos com Baixa Cobertura (MÉDIA)

#### 1. `src/application/` (média: ~60%)
- **`build_use_case.rs`**: 75/83 linhas (90.4%)
- **`create/time_off.rs`**: 10/19 linhas (52.6%)
- **`create/vacation.rs`**: 15/24 linhas (62.5%)
- **`project/assign_resource_to_task.rs`**: 16/19 linhas (84.2%)
- **`report/task.rs`**: 27/55 linhas (49.1%)
- **`task/link_task.rs`**: 25/33 linhas (75.8%)
- **`task/remove_dependency.rs`**: 24/39 linhas (61.5%)
- **`validate/vacations.rs`**: 22/47 linhas (46.8%)

#### 2. `src/domain/` (média: ~50%)
- **`project_management/any_project.rs`**: 136/239 linhas (56.9%)
- **`project_management/builder.rs`**: 23/89 linhas (25.8%)
- **`project_management/project.rs`**: 69/104 linhas (66.3%)
- **`resource_management/any_resource.rs`**: 36/74 linhas (48.6%)
- **`resource_management/resource.rs`**: 63/111 linhas (56.8%)
- **`task_management/any_task.rs`**: 34/87 linhas (39.1%)
- **`task_management/task.rs`**: 119/143 linhas (83.2%)

#### 3. `src/infrastructure/` (média: ~70%)
- **`persistence/config_repository.rs`**: 9/26 linhas (34.6%)
- **`persistence/project_repository.rs`**: 59/100 linhas (59.0%)
- **`persistence/resource_repository.rs`**: 87/126 linhas (69.0%)
- **`persistence/task_repository.rs`**: 48/72 linhas (66.7%)

## 🚀 Plano de Implementação

### Fase 1: Testes Unitários Críticos (Dias 1-4)

#### Dia 1: Domain Errors e Shared Traits
1. **`src/domain/shared/errors.rs`** - Testar todos os métodos e cenários de erro
2. **`src/domain/shared/validatable.rs`** - Testar trait de validação
3. **`src/domain/shared/convertable.rs`** - Testar conversões

#### Dia 2: Domain Entities e States
1. **`src/domain/project_management/builder.rs`** - Testar todos os cenários de construção
2. **`src/domain/project_management/any_project.rs`** - Testar métodos não cobertos
3. **`src/domain/resource_management/any_resource.rs`** - Testar métodos não cobertos

#### Dia 3: Application Layer
1. **`src/application/create/time_off.rs`** - Testar cenários de erro e edge cases
2. **`src/application/create/vacation.rs`** - Testar validações e cenários de erro
3. **`src/application/report/task.rs`** - Testar formatos de saída e edge cases

#### Dia 4: Infrastructure Errors
1. **`src/infrastructure/errors.rs`** - Testar todos os tipos de erro
2. **`src/domain/company_settings/errors.rs`** - Testar erros de configuração
3. **`src/domain/project_management/errors.rs`** - Testar erros de projeto

### Fase 2: Testes de Integração (Dias 5-7)

#### Dia 5: Repository Integration Tests
1. **`src/infrastructure/persistence/config_repository.rs`** - Testes com diretórios temporários
2. **`src/infrastructure/persistence/project_repository.rs`** - Testes de concorrência e corrupção
3. **`src/infrastructure/persistence/resource_repository.rs`** - Testes de edge cases

#### Dia 6: Persistence Edge Cases
1. **`src/infrastructure/persistence/task_repository.rs`** - Testes de dependências e relacionamentos
2. **`src/infrastructure/persistence/manifests/`** - Testes de serialização/deserialização
3. **Testes de corrupção de arquivos** - YAML inválido, permissões, etc.

#### Dia 7: Application Integration
1. **`src/application/build_use_case.rs`** - Testes de construção completa
2. **`src/application/validate/vacations.rs`** - Testes de validação complexa
3. **`src/application/task/link_task.rs`** - Testes de dependências circulares

### Fase 3: Testes CLI e End-to-End (Dias 8-10)

#### Dia 8: CLI Unit Tests
1. **`src/interface/cli.rs`** - Testar parsing de comandos
2. **Testar argumentos válidos e inválidos**
3. **Testar subcomandos e opções**

#### Dia 9: CLI Integration Tests
1. **Testes de comandos completos** - `init`, `create`, `list`, `report`
2. **Testes de arquivos temporários** - Usar `tempfile` para persistência
3. **Testes de saída** - CSV, formatação, etc.

#### Dia 10: End-to-End Scenarios
1. **Fluxo completo de projeto** - Criar, modificar, relatar
2. **Fluxo de recursos** - Criar, atribuir, gerenciar férias
3. **Fluxo de tarefas** - Criar, vincular, executar

## 🛠️ Ferramentas e Configuração

### 1. Configuração do Cargo
```toml
[dev-dependencies]
tempfile = "3.8"
assert_fs = "1.1"
test-case = "3.3"
mockall = "0.12"
```

### 2. Estrutura de Testes
```
tests/
├── unit/           # Testes unitários isolados
├── integration/    # Testes de integração
├── e2e/           # Testes end-to-end
└── fixtures/      # Dados de teste
```

### 3. Testes com Diretórios Temporários
```rust
use tempfile::TempDir;
use std::fs;

#[test]
fn test_with_temp_directory() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    
    // Testar funcionalidade
    // TempDir é automaticamente limpo
}
```

## 📈 Métricas de Progresso

### Indicadores de Sucesso
- **Cobertura de Linhas**: 85%+
- **Cobertura de Branches**: 80%+
- **Testes Passando**: 100%
- **Tempo de Execução**: <30 segundos

### Checkpoints Diários
- **Dia 3**: 60% de cobertura
- **Dia 7**: 75% de cobertura
- **Dia 10**: 85% de cobertura

## 🔍 Áreas de Foco Específicas

### 1. Error Handling
- Testar todos os cenários de erro
- Validar mensagens de erro
- Testar propagação de erros

### 2. Edge Cases
- Dados inválidos
- Arquivos corrompidos
- Concorrência
- Falhas de I/O

### 3. CLI Commands
- Argumentos válidos/inválidos
- Subcomandos
- Formatação de saída
- Tratamento de erros

### 4. Persistence
- Serialização/deserialização
- Corrupção de arquivos
- Concorrência
- Permissões

## 🚨 Riscos e Mitigações

### Riscos Identificados
1. **Complexidade dos testes CLI** - Muitos cenários para cobrir
2. **Testes de concorrência** - Difícil de reproduzir
3. **Dependências externas** - Arquivos, sistema de arquivos

### Mitigações
1. **Testes incrementais** - Começar simples, evoluir
2. **Mocks e stubs** - Isolar dependências
3. **Testes determinísticos** - Evitar flakiness

## 📝 Próximos Passos Imediatos

1. **Configurar dependências de teste**
2. **Criar estrutura de diretórios de teste**
3. **Implementar testes de erro básicos**
4. **Executar análise de cobertura diária**

---

**Estimativa Total**: 10 dias
**Esforço**: Alto
**Benefício**: Código robusto e confiável
**ROI**: Alto (qualidade, manutenibilidade, confiança)
