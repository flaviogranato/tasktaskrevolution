# 📋 **SISTEMA DE REUNIÕES PROPOSTO PARA TTR**

## 🎯 **Análise Estratégica: Registro de Reuniões no TTR**

### ✅ **SIM, é INTERESSANTE e ESTRATÉGICO ter Registro de Reuniões**

Como seu **point of contact em design de funcionalidades** e especialista em arquitetura, analisei seu sistema e confirmei: **SIM, faz TOTAL sentido** implementar um sistema de reuniões no TTR, especialmente para o público-alvo (gerentes de projeto).

---

## 💡 **Por que Implementar Sistema de Reuniões?**

### **1. Alinhamento com Arquitetura Existente**
Você já tem na base de código:
- ✅ `DailyStandup` - Reuniões diárias ágeis (linha 155-164 em agile_models.rs)
- ✅ `SprintRetrospective` - Retrospectivas com action items (linha 168-177)
- ✅ `ActionItem` - Item de ação com owner e status (linha 181-188)
- ✅ `Comment` em tasks - Comentários com autor e timestamp
- ✅ `GlobalSchedule` - Cronograma global com participantes (linha 162-207)

**O sistema de reuniões seria uma EVOLUÇÃO NATURAL dessas funcionalidades existentes.**

### **2. Valor para Gerentes de Projeto**
- 📊 **Rastreamento de Decisões**: Decisões importantes ficam registradas
- 📝 **Transparência**: Atas disponíveis para toda equipe
- ✅ **Accountability**: Action items com donos claros
- 🔗 **Integração**: Link entre reuniões → tasks → projetos
- 📈 **Métricas**: Análise de produtividade de reuniões

### **3. Gaps Atuais Identificados**
- ❌ Não há **repositório centralizado** de reuniões
- ❌ Não há **ligação entre decisões e tasks** criadas
- ❌ Não há **atas estruturadas** fora de contextos ágeis
- ❌ Não há **search de decisões** anteriores
- ❌ Não há **follow-up automático** de action items

---

## 🏗️ **Design Proposto: Sistema de Reuniões**

### **Arquitetura**

```
domain/
  ├── meetings/              # Novo módulo de reuniões
  │   ├── meeting.rs         # Entidade Meeting
  │   ├── decision.rs        # Entidade Decision
  │   ├── meeting_manager.rs  # Gestor de reuniões
  │   ├── meeting_types.rs   # Tipos de reunião
  │   └── mod.rs
```

### **Modelos de Dados**

```rust
// Tipos de reunião
pub enum MeetingType {
    SprintPlanning,      // Já existe em agile
    DailyStandup,        // Já existe em agile
    SprintReview,        // Já existe em agile
    Retrospective,       // Já existe em agile
    OneOnOne,            // Novo!
    TeamMeeting,         // Novo!
    ProjectReview,       // Novo!
    StakeholderReview,   // Novo!
    DecisionMaking,      // Novo!
    BlockerResolution,   // Novo!
    Custom(String),       // Reunião customizada
}

// Entidade Reunião
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meeting {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub meeting_type: MeetingType,
    
    // Agendamento
    pub scheduled_at: DateTime<Utc>,
    pub duration_minutes: i32,
    pub timezone: Option<String>,
    
    // Contexto
    pub project_id: Option<String>,
    pub sprint_id: Option<String>,
    pub task_id: Option<String>,
    
    // Participantes
    pub organizer_id: String,
    pub participants: Vec<Participant>,
    pub required_attendees: Vec<String>,
    pub optional_attendees: Vec<String>,
    
    // Conteúdo
    pub agenda: Vec<AgendaItem>,
    pub minutes: Vec<Minute>,
    pub decisions: Vec<Decision>,
    pub action_items: Vec<ActionItem>,
    pub attachments: Vec<Attachment>,
    
    // Status
    pub status: MeetingStatus,
    pub actual_start: Option<DateTime<Utc>>,
    pub actual_end: Option<DateTime<Utc>>,
    pub attendance_actual: Vec<String>, // Quem compareceu
    
    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: String,
}

// Participante
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub user_id: String,
    pub name: String,
    pub role: Option<String>,
    pub attendance: AttendanceStatus,
    pub contribution_notes: Option<String>,
}

// Status de presença
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttendanceStatus {
    Confirmed,
    Declined,
    Tentative,
    Present,
    Absent,
    Late,
}

// Status da reunião
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MeetingStatus {
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
    Rescheduled,
}

// Item de agenda
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgendaItem {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub presenter_id: Option<String>,
    pub time_allotted_minutes: Option<i32>,
    pub status: AgendaItemStatus,
}

// Decisão
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: String,
    pub title: String,
    pub description: String,
    pub rationale: Option<String>, // Por que foi decidido
    pub decided_by: String, // Quem decidiu
    pub decision_date: DateTime<Utc>,
    pub effective_date: Option<DateTime<Utc>>, // Quando entra em vigor
    pub review_date: Option<DateTime<Utc>>, // Quando revisar
    pub related_task_ids: Vec<String>,
    pub impact: DecisionImpact,
    pub status: DecisionStatus,
}

// Impacto da decisão
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DecisionImpact {
    Low,      // Impacto baixo
    Medium,   // Impacto médio
    High,     // Impacto alto
    Critical, // Impacto crítico
}

// Status da decisão
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DecisionStatus {
    Proposed,
    Approved,
    Rejected,
    Implemented,
    Superseded,
}

// Ata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Minute {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub author_id: String,
    pub content: String,
    pub topic: Option<String>, // Tópico discutido
    pub speakers: Vec<String>, // Quem falou
}

// Action Item (melhorado)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub id: String,
    pub description: String,
    pub owner: String, // User ID responsável
    pub due_date: Option<DateTime<Utc>>,
    pub status: ActionItemStatus,
    pub priority: ActionItemPriority,
    pub related_task_id: Option<String>, // Pode criar task a partir disso
    pub related_project_id: Option<String>,
    pub completion_notes: Option<String>,
    pub completion_date: Option<DateTime<Utc>>,
}
```

---

## 🎯 **Funcionalidades Propostas**

### **1. Gestão de Reuniões**
- ✅ **Agendar reuniões** por projeto/sprint
- ✅ **Convidar participantes** com confirmação
- ✅ **Definir agenda** estruturada
- ✅ **Registrar decisões** e action items
- ✅ **Enviar atas** automaticamente por email

### **2. Sistema de Decisões**
- ✅ **Rastrear decisões** por projeto
- ✅ **Ligar decisões a tasks** criadas
- ✅ **Revisar decisões** periodicamente
- ✅ **Buscar decisões históricas**
- ✅ **Analisar impacto** de decisões

### **3. Action Items**
- ✅ **Criar action items** durante reunião
- ✅ **Atribuir responsáveis** com prazos
- ✅ **Converter em tasks** automaticamente
- ✅ **Follow-up automático** de pendências
- ✅ **Relatórios de cumprimento**

### **4. Integração com Domínios Existentes**
- 🔗 **Projetos**: Reuniões vinculadas a projetos
- 🔗 **Sprints**: Reuniões vinculadas a sprints
- 🔗 **Tasks**: Action items viram tasks
- 🔗 **Recursos**: Participantes são recursos
- 🔗 **Timeline**: Reuniões no cronograma

---

## 💻 **CLI Commands Propostos**

```bash
# Gestão de reuniões
ttr meeting schedule          # Agendar reunião
ttr meeting list              # Listar reuniões
ttr meeting show <id>         # Mostrar detalhes
ttr meeting cancel <id>       # Cancelar reunião
ttr meeting reschedule <id>   # Reagendar

# Durante reunião
ttr meeting start <id>        # Iniciar reunião
ttr meeting add-decision      # Adicionar decisão
ttr meeting add-action-item   # Adicionar action item
ttr meeting add-minute        # Adicionar ata
ttr meeting end <id>          # Finalizar reunião

# Decisões
ttr decision list              # Listar decisões
ttr decision show <id>        # Mostrar decisão
ttr decision search <query>   # Buscar decisões
ttr decision review <id>       # Revisar decisão

# Action Items
ttr action-item list           # Listar action items
ttr action-item create-task   # Converter em task
ttr action-item follow-up      # Ver pendências
ttr action-item complete <id> # Marcar completo

# Relatórios
ttr meeting report             # Relatório de reuniões
ttr decision report            # Relatório de decisões
ttr action-item report         # Relatório de action items
```

---

## 📊 **Valor Estratégico**

### **Para Gerentes de Projeto**
1. **Rastreamento completo** de decisões e ação
2. **Accountability** clara de responsabilidades
3. **Histórico** para auditoria e compliance
4. **Eficiência** com follow-up automático
5. **Integração** com workflow existente

### **Para o TTR**
1. **Diferenciação** de concorrentes
2. **Completude** do escopo de PM
3. **Integração rica** entre domínios
4. **Valor agregado** para cliente
5. **Mais sticky** (usuários ficam mais)

---

## 🚀 **Recomendação Final**

### **✅ IMPLEMENTAR com foco em:**
1. **Decisões rastreáveis** (maior valor para PM)
2. **Action items** vinculados a tasks
3. **Atas estruturadas** com templates
4. **Integração profunda** com projetos/sprints
5. **CLI-first** mantendo filosofia do TTR

### **Priorização**
- **P1**: Funcionalidades core (agenda, ata, decisões, action items)
- **P2**: Integração com domínios existentes
- **P3**: Templates avançados e automações

### **MVP Sugerido**
- Criar reunião com agenda básica
- Registrar decisões simples
- Criar action items
- Linkar com projeto/sprint
- Gerar ata em markdown

---

## 📝 **Próximos Passos**

1. Validar se faz sentido criar issue no GitHub
2. Definir escopo MVP para implementação
3. Priorizar em roadmap (sugiro após Fase 1-2)
4. Projetar detalhadamente modelos de dados
5. Implementar em sprint dedicada

**CONCLUSÃO**: Sistema de reuniões é **strategic fit** perfeito para TTR e complementa sua visão de ser substituto do MS Project com foco em CLI! 🎯

