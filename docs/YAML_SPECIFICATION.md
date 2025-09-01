# TaskTaskRevolution YAML Specification

## Overview

TaskTaskRevolution (TTR) utiliza um padrão de YAML inspirado no Kubernetes e Backstage para representar entidades do sistema de forma consistente e estruturada. Este padrão garante que todos os recursos do sistema sigam a mesma estrutura, facilitando a manutenção, validação e interoperabilidade.

## Estrutura Base

Todos os YAMLs do TTR seguem esta estrutura padrão:

```yaml
apiVersion: <resource-type>.tasktaskrevolution.io/v1
kind: <ResourceType>
metadata:
  # Identificação e metadados da entidade
spec:
  # Especificação e dados da entidade
```

### Campos Obrigatórios

- **`apiVersion`**: Versão da API do recurso (formato: `{resource-type}.tasktaskrevolution.io/v1`)
- **`kind`**: Tipo da entidade (ex: `Company`, `Project`, `Resource`, `Task`)
- **`metadata`**: Seção obrigatória contendo identificadores e metadados
- **`spec`**: Seção obrigatória contendo os dados específicos da entidade

## Especificação por Tipo de Recurso

### 1. Company (Empresa)

**apiVersion**: `company.tasktaskrevolution.io/v1`  
**kind**: `Company`

#### Estrutura

```yaml
apiVersion: company.tasktaskrevolution.io/v1
kind: Company
metadata:
  id: string                    # UUID v7 único global
  code: string                  # Código sequencial (ex: "COMP-001")
  name: string                  # Nome da empresa
  createdAt: datetime           # Data/hora de criação (ISO 8601)
  updatedAt: datetime           # Data/hora da última atualização (ISO 8601)
  createdBy: string             # Usuário que criou a entidade
spec:
  description: string?          # Descrição opcional
  taxId: string?               # CNPJ/CPF
  address: string?             # Endereço completo
  email: string?               # Email de contato
  phone: string?               # Telefone
  website: string?             # Website
  industry: string?            # Setor/indústria
  size: CompanySize            # Tamanho da empresa
  status: CompanyStatus        # Status atual
```

#### Enums

**CompanySize**:
- `small` - Pequena (1-50 funcionários)
- `medium` - Média (51-250 funcionários)
- `large` - Grande (251+ funcionários)

**CompanyStatus**:
- `active` - Ativa
- `inactive` - Inativa
- `suspended` - Suspensa

#### Exemplo

```yaml
apiVersion: company.tasktaskrevolution.io/v1
kind: Company
metadata:
  id: "01990605-fbf1-7609-9a45-6586c303be8a"
  code: "COMP-001"
  name: "TechConsulting Ltda"
  createdAt: "2025-09-01T16:04:39.537860402Z"
  updatedAt: "2025-09-01T16:04:39.537940342Z"
  createdBy: "flavio@example.com"
spec:
  description: "Empresa de consultoria em TI"
  taxId: "12.345.678/0001-90"
  address: "Rua das Flores, 123, São Paulo, SP"
  email: "contato@techconsulting.com"
  phone: "(11) 99999-9999"
  website: "www.techconsulting.com"
  industry: "Tecnologia"
  size: "medium"
  status: "active"
```

### 2. Project (Projeto)

**apiVersion**: `project.tasktaskrevolution.io/v1`  
**kind**: `Project`

#### Estrutura

```yaml
apiVersion: project.tasktaskrevolution.io/v1
kind: Project
metadata:
  id: string                    # UUID v7 único global
  code: string                  # Código sequencial (ex: "PROJ-001")
  name: string                  # Nome do projeto
  createdAt: datetime           # Data/hora de criação
  updatedAt: datetime           # Data/hora da última atualização
  createdBy: string             # Usuário que criou o projeto
spec:
  description: string?          # Descrição do projeto
  companyCode: string           # Código da empresa proprietária
  startDate: date               # Data de início
  endDate: date                 # Data de término planejada
  actualEndDate: date?          # Data de término real (se concluído)
  priority: ProjectPriority     # Prioridade do projeto
  status: ProjectStatus         # Status atual
  budget: number?               # Orçamento em reais
  manager: string?              # Gerente do projeto
  tags: string[]?               # Tags para categorização
```

#### Enums

**ProjectPriority**:
- `low` - Baixa
- `medium` - Média
- `high` - Alta
- `critical` - Crítica

**ProjectStatus**:
- `planned` - Planejado
- `in_progress` - Em andamento
- `completed` - Concluído
- `cancelled` - Cancelado
- `on_hold` - Em espera

### 3. Resource (Recurso)

**apiVersion**: `resource.tasktaskrevolution.io/v1`  
**kind**: `Resource`

#### Estrutura

```yaml
apiVersion: resource.tasktaskrevolution.io/v1
kind: Resource
metadata:
  id: string                    # UUID v7 único global
  code: string                  # Código sequencial (ex: "DEV-001")
  name: string                  # Nome do recurso
  createdAt: datetime           # Data/hora de criação
  updatedAt: datetime           # Data/hora da última atualização
  createdBy: string             # Usuário que criou o recurso
spec:
  email: string?                # Email do recurso
  resourceType: ResourceType    # Tipo de recurso
  skills: string[]?             # Habilidades técnicas
  hourlyRate: number?           # Taxa horária em reais
  availability: Availability    # Disponibilidade
  vacations: Period[]?          # Períodos de férias
  timeOffBalance: number        # Saldo de folgas (em horas)
```

#### Enums

**ResourceType**:
- `developer` - Desenvolvedor
- `designer` - Designer
- `manager` - Gerente
- `analyst` - Analista
- `tester` - Testador
- `devops` - DevOps

**Availability**:
- `available` - Disponível
- `busy` - Ocupado
- `unavailable` - Indisponível
- `part_time` - Tempo parcial

### 4. Task (Tarefa)

**apiVersion**: `task.tasktaskrevolution.io/v1`  
**kind**: `Task`

#### Estrutura

```yaml
apiVersion: task.tasktaskrevolution.io/v1
kind: Task
metadata:
  id: string                    # UUID v7 único global
  code: string                  # Código sequencial (ex: "TASK-001")
  name: string                  # Nome da tarefa
  createdAt: datetime           # Data/hora de criação
  updatedAt: datetime           # Data/hora da última atualização
  createdBy: string             # Usuário que criou a tarefa
spec:
  projectCode: string           # Código do projeto
  description: string?          # Descrição da tarefa
  startDate: date               # Data de início
  dueDate: date                 # Data de vencimento
  actualEndDate: date?          # Data de conclusão real
  estimatedHours: number        # Horas estimadas
  actualHours: number?          # Horas reais trabalhadas
  priority: TaskPriority        # Prioridade da tarefa
  status: TaskStatus            # Status atual
  dependencies: string[]?       # Códigos das tarefas dependentes
  assignedResources: string[]?  # Códigos dos recursos designados
  tags: string[]?               # Tags para categorização
```

#### Enums

**TaskPriority**:
- `low` - Baixa
- `medium` - Média
- `high` - Alta
- `urgent` - Urgente

**TaskStatus**:
- `todo` - A fazer
- `in_progress` - Em andamento
- `review` - Em revisão
- `testing` - Em teste
- `done` - Concluída
- `blocked` - Bloqueada

## Convenções e Padrões

### Nomenclatura

- **Códigos**: Formato `{TIPO}-{NUMERO}` (ex: `COMP-001`, `PROJ-001`)
- **IDs**: UUID v7 para identificação global única
- **Campos**: camelCase para metadados, snake_case para especificações
- **Datas**: Formato ISO 8601 (UTC)

### Validação

- Campos obrigatórios devem estar sempre presentes
- Campos opcionais podem ser `null` ou omitidos
- Enums devem conter apenas valores válidos
- Datas devem estar no formato correto
- UUIDs devem ser válidos

### Versionamento

- **v1**: Versão atual da especificação
- Mudanças incompatíveis requerem nova versão
- Versões antigas são suportadas durante transição

## Estrutura de Diretórios

```
.
├── companies/                 # Entidades Company
│   ├── COMP-001.yaml
│   ├── COMP-002.yaml
│   └── ...
├── projects/                  # Entidades Project
│   ├── PROJ-001.yaml
│   ├── PROJ-002.yaml
│   └── ...
├── resources/                 # Entidades Resource
│   ├── DEV-001.yaml
│   ├── DEV-002.yaml
│   └── ...
├── tasks/                     # Entidades Task
│   ├── TASK-001.yaml
│   ├── TASK-002.yaml
│   └── ...
└── config.yaml                # Configuração global
```

## Benefícios do Padrão

1. **Consistência**: Estrutura uniforme para todas as entidades
2. **Validação**: Schema validation automático
3. **Interoperabilidade**: Compatível com ferramentas YAML
4. **Manutenibilidade**: Fácil de entender e modificar
5. **Extensibilidade**: Novos campos podem ser adicionados sem quebrar compatibilidade
6. **Auditoria**: Rastreamento completo de mudanças via timestamps
7. **Versionamento**: Controle de versão da API

## Ferramentas Compatíveis

- **Editores**: VS Code, Vim, Emacs
- **Validadores**: yamllint, yq
- **Processadores**: jq, yq, kubectl
- **IDEs**: IntelliJ, Eclipse
- **Git**: Controle de versão nativo

## Exemplos de Uso

### Criação via CLI

```bash
# Criar empresa
ttr company create --code "COMP-001" --name "Minha Empresa" --created-by "user@example.com"

# Criar projeto
ttr project create --code "PROJ-001" --name "Meu Projeto" --company-code "COMP-001" --created-by "user@example.com"

# Criar recurso
ttr resource create --code "DEV-001" --name "João Silva" --type "developer" --created-by "user@example.com"
```

### Edição Manual

```bash
# Editar YAML diretamente
vim companies/COMP-001.yaml

# Validar sintaxe
yamllint companies/COMP-001.yaml

# Extrair informações
yq '.metadata.name' companies/COMP-001.yaml
```

## Considerações de Segurança

- **IDs únicos**: UUID v7 previne enumeração
- **Validação**: Todos os campos são validados antes da persistência
- **Sanitização**: Entrada do usuário é sempre sanitizada
- **Auditoria**: Todas as mudanças são registradas com timestamp e usuário

## Roadmap

- [x] Especificação Company (v1)
- [ ] Especificação Project (v1)
- [ ] Especificação Resource (v1)
- [ ] Especificação Task (v1)
- [ ] Validação de Schema
- [ ] Migração de Dados
- [ ] Documentação de API
- [ ] Ferramentas de Validação

---

*Esta especificação é inspirada nos padrões do Kubernetes e Backstage, adaptada para as necessidades específicas do TaskTaskRevolution.*
