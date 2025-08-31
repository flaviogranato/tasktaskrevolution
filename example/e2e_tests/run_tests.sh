#!/bin/bash

# Script para executar testes E2E do TTR CLI
# Uso: ./run_tests.sh [test_name]

set -e

# Cores para output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Função para imprimir com cores
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Verificar se estamos no diretório correto
if [ ! -f "Cargo.toml" ]; then
    print_error "Este script deve ser executado no diretório e2e_tests/"
    exit 1
fi

# Verificar se o TTR CLI está compilado
if [ ! -f "../../target/release/ttr" ]; then
    print_warning "TTR CLI não encontrado. Compilando..."
    cd ../..
    cargo build --release
    cd example/e2e_tests
fi

# Função para executar testes específicos
run_specific_test() {
    local test_name=$1
    print_info "Executando teste: $test_name"
    
    case $test_name in
        "project_lifecycle")
            cargo test --test project_lifecycle -- --nocapture
            ;;
        "cli_integration")
            cargo test --test cli_integration -- --nocapture
            ;;
        "file_validation")
            cargo test --test file_validation -- --nocapture
            ;;
        "utilities")
            cargo test --test utilities -- --nocapture
            ;;
        "all")
            cargo test --test e2e -- --nocapture
            ;;
        *)
            print_error "Teste desconhecido: $test_name"
            print_info "Testes disponíveis: project_lifecycle, cli_integration, file_validation, utilities, all"
            exit 1
            ;;
    esac
}

# Função para executar todos os testes
run_all_tests() {
    print_info "Executando todos os testes E2E..."
    
    # Testes de utilitários
    print_info "1. Testando utilitários..."
    cargo test --test utilities -- --nocapture
    
    # Testes de integração CLI
    print_info "2. Testando integração CLI..."
    cargo test --test cli_integration -- --nocapture
    
    # Testes de validação de arquivos
    print_info "3. Testando validação de arquivos..."
    cargo test --test file_validation -- --nocapture
    
    # Testes de ciclo de vida do projeto
    print_info "4. Testando ciclo de vida do projeto..."
    cargo test --test project_lifecycle -- --nocapture
    
    # Teste principal E2E
    print_info "5. Executando teste principal E2E..."
    cargo test --test e2e -- --nocapture
    
    print_success "Todos os testes foram executados com sucesso!"
}

# Função para mostrar ajuda
show_help() {
    echo "Uso: $0 [OPÇÃO]"
    echo ""
    echo "Opções:"
    echo "  project_lifecycle  Executa testes de ciclo de vida do projeto"
    echo "  cli_integration    Executa testes de integração CLI"
    echo "  file_validation    Executa testes de validação de arquivos"
    echo "  utilities          Executa testes de utilitários"
    echo "  all                Executa todos os testes"
    echo "  help               Mostra esta ajuda"
    echo ""
    echo "Exemplos:"
    echo "  $0 project_lifecycle"
    echo "  $0 all"
    echo "  $0"
}

# Função principal
main() {
    print_info "🚀 Iniciando testes E2E do TTR CLI..."
    
    # Verificar argumentos
    if [ $# -eq 0 ]; then
        # Sem argumentos, executar todos os testes
        run_all_tests
    elif [ "$1" = "help" ] || [ "$1" = "-h" ] || [ "$1" = "--help" ]; then
        show_help
    else
        # Executar teste específico
        run_specific_test "$1"
    fi
}

# Executar função principal
main "$@"
