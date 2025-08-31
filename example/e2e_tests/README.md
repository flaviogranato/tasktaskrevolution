# Testes E2E para TTR CLI

Este diretório contém testes end-to-end para validar o funcionamento completo do CLI TTR.

## 🎯 Objetivos

- Validar fluxos completos de criação, edição e consulta de projetos
- Garantir integridade entre diferentes comandos
- Testar cenários reais de uso por consultores de projetos
- Verificar persistência e consistência dos dados YAML

## 📁 Estrutura

```
e2e_tests/
├── fixtures/           # Dados de teste pré-definidos
│   ├── sample_project.yaml
│   ├── sample_company.yaml
│   └── sample_resources.yaml
├── scenarios/          # Cenários de teste
│   ├── project_lifecycle.rs
│   ├── resource_management.rs
│   ├── reporting_workflow.rs
│   └── error_handling.rs
├── utils/              # Utilitários para testes
│   ├── cli_runner.rs
│   ├── file_assertions.rs
│   └── yaml_validator.rs
└── integration/        # Testes de integração
    ├── yaml_persistence.rs
    ├── command_chaining.rs
    └── data_consistency.rs
```

## 🚀 Como Executar

```bash
# Executar todos os testes e2e
cargo test --package TaskTaskRevolution --test e2e

# Executar cenário específico
cargo test --package TaskTaskRevolution --test project_lifecycle

# Executar com output detalhado
cargo test --package TaskTaskRevolution --test e2e -- --nocapture
```

## 📋 Cenários de Teste

### 1. Project Lifecycle
- Criação de projeto
- Adição de tarefas
- Mudança de status
- Finalização de projeto

### 2. Resource Management
- Criação de recursos
- Atribuição a tarefas
- Detecção de conflitos
- Leveling automático

### 3. Reporting Workflow
- Geração de dashboards
- Exportação CSV
- Geração de Gantt
- Análise de métricas

### 4. Error Handling
- Validações de entrada
- Tratamento de erros
- Recuperação de falhas
- Logs e mensagens

## 🔧 Configuração

Os testes usam arquivos temporários e diretórios isolados para garantir:
- Isolamento entre testes
- Limpeza automática
- Repetibilidade
- Independência de estado
