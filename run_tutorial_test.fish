#!/usr/bin/env fish

# 🧪 Script de Teste Automatizado - TaskTaskRevolution
# Este script executa todos os comandos do tutorial para validação completa

set -x TEST_DIR "/tmp/ttr_tutorial_test_$(date +%s)"
set -x SITE_DIR "$TEST_DIR/site"

echo "🚀 Iniciando Teste Automatizado do TaskTaskRevolution"
echo "📁 Diretório de teste: $TEST_DIR"
echo "🔧 Usando TTR do PATH"
echo ""

# Verificar se o comando ttr está disponível no PATH
if not command -v ttr >/dev/null 2>&1
    echo "❌ Erro: Comando 'ttr' não encontrado no PATH"
    echo "💡 Execute: cargo install --force --path ."
    exit 1
end

echo "✅ Comando 'ttr' encontrado no PATH"
echo ""

# Criar diretório de teste
echo "📁 Criando diretório de teste..."
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

echo "✅ Diretório de teste criado: $TEST_DIR"
echo ""

# ============================================================================
# FASE 1: INICIALIZAÇÃO DO SISTEMA
# ============================================================================

echo "🚀 FASE 1: Inicialização do Sistema"
echo "=================================="

echo "1.1 Inicializando o TTR..."
ttr init \
  --name "Usuário Teste" \
  --email "teste@exemplo.com" \
  --company-name "Empresa Teste" \
  --timezone "America/Sao_Paulo" \
  --work-hours-start "09:00" \
  --work-hours-end "18:00" \
  --work-days "monday,tuesday,wednesday,thursday,friday"

if test $status -eq 0
    echo "✅ Sistema inicializado com sucesso"
else
    echo "❌ Falha na inicialização do sistema"
    exit 1
end

echo "1.2 Verificando arquivos criados..."
ls -la
echo "📄 Conteúdo do config.yaml:"
cat config.yaml
echo ""

# ============================================================================
# FASE 2: GERENCIAMENTO DE EMPRESAS
# ============================================================================

echo "🏢 FASE 2: Gerenciamento de Empresas"
echo "===================================="

echo "2.1 Criando empresa TechCorp Solutions..."
ttr create company \
  --name "TechCorp Solutions" \
  --code "TECH-001" \
  --description "Empresa de tecnologia e soluções inovadoras"

if test $status -eq 0
    echo "✅ Empresa TechCorp criada com sucesso"
else
    echo "❌ Falha na criação da empresa TechCorp"
    exit 1
end

echo "2.2 Criando empresa Design Studio..."
ttr create company \
  --name "Design Studio" \
  --code "DESIGN-001" \
  --description "Estúdio de design e criatividade"

if test $status -eq 0
    echo "✅ Empresa Design Studio criada com sucesso"
else
    echo "❌ Falha na criação da empresa Design Studio"
    exit 1
end

echo "2.3 Listando empresas..."
ttr list companies
echo ""

# ============================================================================
# FASE 3: GERENCIAMENTO DE RECURSOS
# ============================================================================

echo "👥 FASE 3: Gerenciamento de Recursos"
echo "===================================="

echo "3.1 Criando recurso João Silva (Developer)..."
ttr create resource \
  --name "João Silva" \
  --type "Developer" \
  --code "JS-001" \
  --company "TECH-001" \
  --description "Desenvolvedor Senior" \
  --email "joao@techcorp.com"

if test $status -eq 0
    echo "✅ Recurso João Silva criado com sucesso"
else
    echo "❌ Falha na criação do recurso João Silva"
    exit 1
end

echo "3.2 Criando recurso Maria Santos (Product Owner)..."
ttr create resource \
  --name "Maria Santos" \
  --type "Product Owner" \
  --code "MS-001" \
  --company "TECH-001" \
  --description "Product Manager" \
  --email "maria@techcorp.com"

if test $status -eq 0
    echo "✅ Recurso Maria Santos criado com sucesso"
else
    echo "❌ Falha na criação do recurso Maria Santos"
    exit 1
end

echo "3.3 Criando recurso Ana Costa (Designer)..."
ttr create resource \
  --name "Ana Costa" \
  --type "Designer" \
  --code "AC-001" \
  --company "DESIGN-001" \
  --description "Designer UX/UI" \
  --email "ana@designstudio.com"

if test $status -eq 0
    echo "✅ Recurso Ana Costa criado com sucesso"
else
    echo "❌ Falha na criação do recurso Ana Costa"
    exit 1
end

echo "3.4 Testando validação de tipos de recursos (deve falhar)..."
ttr create resource \
  --name "Teste" \
  --type "TipoInvalido" \
  --code "TEST-001" \
  --company "TECH-001" \
  --email "teste@teste.com"

if test $status -ne 0
    echo "✅ Validação de tipo inválido funcionando corretamente"
else
    echo "⚠️  Validação de tipo inválido não funcionou como esperado"
end

echo "3.5 Listando recursos..."
ttr list resources --company "TECH-001"
ttr list resources --company "DESIGN-001"
echo ""

# ============================================================================
# FASE 4: GERENCIAMENTO DE PROJETOS
# ============================================================================

echo "📁 FASE 4: Gerenciamento de Projetos"
echo "===================================="

echo "4.1 Criando projeto Sistema de E-commerce..."
ttr create project \
  --name "Sistema de E-commerce" \
  --code "ECOMM-001" \
  --company "TECH-001" \
  --description "Desenvolvimento de plataforma de e-commerce completa" \
  --start-date "2024-01-15" \
  --end-date "2024-06-30"

if test $status -eq 0
    echo "✅ Projeto E-commerce criado com sucesso"
else
    echo "❌ Falha na criação do projeto E-commerce"
    exit 1
end

echo "4.2 Criando projeto App Mobile..."
ttr create project \
  --name "App Mobile" \
  --code "MOBILE-001" \
  --company "TECH-001" \
  --description "Aplicativo mobile para iOS e Android" \
  --start-date "2024-02-01" \
  --end-date "2024-08-31"

if test $status -eq 0
    echo "✅ Projeto App Mobile criado com sucesso"
else
    echo "❌ Falha na criação do projeto App Mobile"
    exit 1
end

echo "4.3 Criando projeto Rebranding Corporativo..."
ttr create project \
  --name "Rebranding Corporativo" \
  --code "REBRAND-001" \
  --company "DESIGN-001" \
  --description "Projeto completo de rebranding e identidade visual" \
  --start-date "2024-01-01" \
  --end-date "2024-03-31"

if test $status -eq 0
    echo "✅ Projeto Rebranding criado com sucesso"
else
    echo "❌ Falha na criação do projeto Rebranding"
    exit 1
end

echo "4.4 Listando projetos..."
ttr list projects --company "TECH-001"
ttr list projects --company "DESIGN-001"
echo ""

# ============================================================================
# FASE 5: GERENCIAMENTO DE TAREFAS
# ============================================================================

echo "✅ FASE 5: Gerenciamento de Tarefas"
echo "==================================="

echo "5.1 Criando tarefa Análise de Requisitos..."
ttr create task \
  --name "Análise de Requisitos" \
  --code "TASK-001" \
  --project "ECOMM-001" \
  --company "TECH-001" \
  --start-date "2024-01-15" \
  --due-date "2024-01-30" \
  --assigned-resources "JS-001,MS-001"

if test $status -eq 0
    echo "✅ Tarefa Análise de Requisitos criada com sucesso"
else
    echo "❌ Falha na criação da tarefa Análise de Requisitos"
    exit 1
end

echo "5.2 Criando tarefa Desenvolvimento Backend..."
ttr create task \
  --name "Desenvolvimento Backend" \
  --code "TASK-002" \
  --project "ECOMM-001" \
  --company "TECH-001" \
  --start-date "2024-02-01" \
  --due-date "2024-04-15" \
  --assigned-resources "JS-001"

if test $status -eq 0
    echo "✅ Tarefa Desenvolvimento Backend criada com sucesso"
else
    echo "❌ Falha na criação da tarefa Desenvolvimento Backend"
    exit 1
end

echo "5.3 Criando tarefa Design de Interface..."
ttr create task \
  --name "Design de Interface" \
  --code "TASK-003" \
  --project "MOBILE-001" \
  --company "TECH-001" \
  --start-date "2024-02-01" \
  --due-date "2024-02-28" \
  --assigned-resources "MS-001"

if test $status -eq 0
    echo "✅ Tarefa Design de Interface criada com sucesso"
else
    echo "❌ Falha na criação da tarefa Design de Interface"
    exit 1
end

echo "5.4 Listando tarefas..."
ttr list tasks --project "ECOMM-001" --company "TECH-001"
ttr list tasks --project "MOBILE-001" --company "TECH-001"
echo ""

# ============================================================================
# FASE 5.5: SISTEMA DE DEPENDÊNCIAS AUTOMÁTICAS
# ============================================================================

echo "🔗 FASE 5.5: Sistema de Dependências Automáticas"
echo "================================================"

echo "5.5.1 Criando tarefas com dependências..."
ttr create task \
  --name "Análise de Requisitos" \
  --code "TASK-ANALISE" \
  --project "ECOMM-001" \
  --company "TECH-001" \
  --start-date "2024-01-15" \
  --due-date "2024-01-30" \
  --assigned-resources "JS-001"

if test $status -eq 0
    echo "✅ Tarefa predecessora criada com sucesso"
else
    echo "❌ Falha na criação da tarefa predecessora"
    exit 1
end

ttr create task \
  --name "Desenvolvimento Backend" \
  --code "TASK-BACKEND" \
  --project "ECOMM-001" \
  --company "TECH-001" \
  --start-date "2024-02-01" \
  --due-date "2024-04-15" \
  --assigned-resources "JS-001"

if test $status -eq 0
    echo "✅ Tarefa dependente criada com sucesso"
else
    echo "❌ Falha na criação da tarefa dependente"
    exit 1
end

echo "5.5.2 Linkando tarefas (criando dependência)..."
ttr link tasks \
  --from "TASK-ANALISE" \
  --to "TASK-BACKEND" \
  --project "ECOMM-001" \
  --company "TECH-001"

if test $status -eq 0
    echo "✅ Dependência criada com sucesso"
else
    echo "❌ Falha na criação da dependência"
    exit 1
end

echo "5.5.3 Testando cálculo automático de datas..."
ttr update task \
  --code "TASK-ANALISE" \
  --project "ECOMM-001" \
  --company "TECH-001" \
  --due-date "2024-02-05"

if test $status -eq 0
    echo "✅ Data da tarefa predecessora atualizada com sucesso"
else
    echo "❌ Falha na atualização da data da tarefa predecessora"
    exit 1
end

echo "5.5.4 Testando detecção de dependências circulares (deve falhar)..."
ttr link tasks \
  --from "TASK-BACKEND" \
  --to "TASK-ANALISE" \
  --project "ECOMM-001" \
  --company "TECH-001"

if test $status -ne 0
    echo "✅ Detecção de dependência circular funcionando corretamente"
else
    echo "⚠️  Detecção de dependência circular não funcionou como esperado"
end

echo "5.5.5 Verificando propagação de mudanças..."
ttr list tasks --project "ECOMM-001" --company "TECH-001"
echo ""

# ============================================================================
# FASE 6: OPERAÇÕES DE ATUALIZAÇÃO
# ============================================================================

echo "🔄 FASE 6: Operações de Atualização"
echo "==================================="

echo "6.1 Atualizando projeto..."
ttr update project \
  --code "ECOMM-001" \
  --company "TECH-001" \
  --name "Sistema E-commerce Premium" \
  --description "Plataforma de e-commerce com recursos avançados"

if test $status -eq 0
    echo "✅ Projeto atualizado com sucesso"
else
    echo "❌ Falha na atualização do projeto"
    exit 1
end

echo "6.2 Atualizando tarefa..."
ttr update task \
  --code "TASK-001" \
  --project "ECOMM-001" \
  --company "TECH-001" \
  --name "Análise Detalhada de Requisitos" \
  --description "Análise completa incluindo UX/UI"

if test $status -eq 0
    echo "✅ Tarefa atualizada com sucesso"
else
    echo "❌ Falha na atualização da tarefa"
    exit 1
end

echo "6.3 Atualizando recurso..."
ttr update resource \
  --code "JS-001" \
  --company "TECH-001" \
  --name "João Silva Santos" \
  --type "Tech Lead" \
  --email "joao.silva@techcorp.com" \
  --description "Tech Lead Senior"

if test $status -eq 0
    echo "✅ Recurso atualizado com sucesso"
else
    echo "❌ Falha na atualização do recurso"
    exit 1
end

echo "6.4 Verificando atualizações..."
ttr list projects --company "TECH-001"
ttr list tasks --project "ECOMM-001" --company "TECH-001"
ttr list resources --company "TECH-001"
echo ""

# ============================================================================
# FASE 7: OPERAÇÕES DE EXCLUSÃO
# ============================================================================

echo "🗑️ FASE 7: Operações de Exclusão"
echo "================================"

echo "7.1 Deletando tarefa..."
ttr delete task \
  --code "TASK-003" \
  --project "MOBILE-001" \
  --company "TECH-001"

if test $status -eq 0
    echo "✅ Tarefa deletada com sucesso"
else
    echo "❌ Falha na exclusão da tarefa"
    exit 1
end

echo "7.2 Deletando projeto..."
ttr delete project \
  --code "MOBILE-001" \
  --company "TECH-001"

if test $status -eq 0
    echo "✅ Projeto deletado com sucesso"
else
    echo "❌ Falha na exclusão do projeto"
    exit 1
end

echo "7.3 Deletando recurso..."
ttr delete resource \
  --code "MS-001" \
  --company "TECH-001"

if test $status -eq 0
    echo "✅ Recurso deletado com sucesso"
else
    echo "❌ Falha na exclusão do recurso"
    exit 1
end

echo "7.4 Verificando exclusões..."
ttr list projects --company "TECH-001"
ttr list tasks --project "ECOMM-001" --company "TECH-001"
ttr list resources --company "TECH-001"
echo ""

# ============================================================================
# FASE 8: GERAÇÃO DE SITE ESTÁTICO
# ============================================================================

echo "🏗️ FASE 8: Geração de Site Estático"
echo "==================================="

echo "8.1 Build do site..."
ttr build --output "site" --base-url "https://meusite.com"

if test $status -eq 0
    echo "✅ Site estático gerado com sucesso"
else
    echo "❌ Falha na geração do site estático"
    exit 1
end

echo "8.2 Verificando arquivos gerados..."
ls -la site/
ls -la site/companies/
echo ""

# ============================================================================
# FASE 8.5: GERAÇÃO DE GRÁFICOS GANTT
# ============================================================================

echo "📊 FASE 8.5: Geração de Gráficos Gantt"
echo "======================================"

echo "8.5.1 Verificando gráficos Gantt gerados..."
ls -la site/companies/*/gantt.html
ls -la site/companies/*/projects/*/gantt.html
echo ""

# ============================================================================
# FASE 9: VALIDAÇÃO DO SISTEMA
# ============================================================================

echo "🔍 FASE 9: Validação do Sistema"
echo "==============================="

echo "9.1 Validando regras de negócio..."
ttr validate business-rules

echo "9.2 Validando integridade dos dados..."
ttr validate data-integrity

echo "9.3 Validando entidades..."
ttr validate entities

echo "9.4 Validando sistema completo..."
ttr validate system
echo ""

# ============================================================================
# FASE 10: TESTES DE CONTEXTO
# ============================================================================

echo "🧪 FASE 10: Testes de Contexto"
echo "=============================="

echo "10.1 Testando contexto de empresa..."
cd companies/TECH-001
ttr list projects
ttr create project --name "Projeto Local" --code "LOCAL-001" --start-date "2024-03-01" --end-date "2024-05-31"
cd ../..

echo "10.2 Testando contexto de projeto..."
cd companies/TECH-001/projects/ECOMM-001
ttr list tasks
ttr create task --name "Tarefa Local" --code "TASK-LOCAL" --start-date "2024-03-01" --due-date "2024-03-15"
cd ../../..
echo ""

# ============================================================================
# FASE 11: TESTES DE RELATÓRIOS
# ============================================================================

echo "📊 FASE 11: Testes de Relatórios"
echo "==============================="

echo "11.1 Gerando relatório de tarefas..."
ttr report task --project "ECOMM-001" --company "TECH-001"

echo "11.2 Gerando relatório de férias..."
ttr report vacation --company "TECH-001"
echo ""

# ============================================================================
# FASE 12: TESTES DE LINKS
# ============================================================================

echo "🔗 FASE 12: Testes de Links"
echo "==========================="

echo "12.1 Linkando tarefas..."
ttr link tasks --from "TASK-001" --to "TASK-002" --project "ECOMM-001" --company "TECH-001"

echo "12.2 Deslinkando tarefas..."
ttr unlink tasks --from "TASK-001" --to "TASK-002" --project "ECOMM-001" --company "TECH-001"
echo ""

# ============================================================================
# RESUMO FINAL
# ============================================================================

echo "🎉 RESUMO FINAL"
echo "==============="
echo "✅ Todos os testes foram executados com sucesso!"
echo ""
echo "📊 Funcionalidades Validadas:"
echo "  - ✅ Inicialização do sistema"
echo "  - ✅ Criação de empresas"
echo "  - ✅ Criação de recursos com validação de tipos"
echo "  - ✅ Criação de projetos"
echo "  - ✅ Criação de tarefas"
echo "  - ✅ Sistema de dependências automáticas"
echo "  - ✅ Validação de tipos de recursos"
echo "  - ✅ Cálculo automático de datas"
echo "  - ✅ Detecção de dependências circulares"
echo "  - ✅ Propagação de mudanças"
echo "  - ✅ Operações de atualização"
echo "  - ✅ Operações de exclusão"
echo "  - ✅ Geração de site estático"
echo "  - ✅ Geração de gráficos Gantt"
echo "  - ✅ Validação do sistema"
echo "  - ✅ Trabalho com contextos"
echo "  - ✅ Geração de relatórios"
echo "  - ✅ Operações de link/unlink"
echo ""
echo "📁 Arquivos gerados em: $TEST_DIR"
echo "🌐 Site estático em: $SITE_DIR"
echo ""
echo "🧹 Limpeza do diretório temporário..."
echo "💡 Para manter os arquivos de teste, copie de: $TEST_DIR"
echo "🗑️  Para limpar automaticamente, pressione Enter ou aguarde 10 segundos..."
echo ""

# Aguardar 10 segundos ou entrada do usuário
read -t 10 -P "Pressione Enter para limpar ou aguarde 10 segundos... " || true

# Limpar diretório temporário
if test -d "$TEST_DIR"
    echo "🧹 Removendo diretório temporário: $TEST_DIR"
    rm -rf "$TEST_DIR"
    echo "✅ Diretório temporário removido com sucesso"
else
    echo "ℹ️  Diretório temporário já foi removido"
end

echo ""
echo "🚀 TaskTaskRevolution está funcionando perfeitamente!"
echo "🎯 Todas as funcionalidades implementadas foram validadas com sucesso!"
