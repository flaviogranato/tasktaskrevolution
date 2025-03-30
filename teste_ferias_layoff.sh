#!/bin/bash

set -e

# Limpar diretório de teste
rm -rf ./teste_ferias
mkdir -p ./teste_ferias
cd ./teste_ferias

echo "1. Inicializando repositório..."
cargo run --quiet -- init . --manager-name "Gerente" --manager-email "gerente@empresa.com"

echo "2. Criando projeto com regras de layoff..."
cat > project.yaml << EOF
apiVersion: tasktaskrevolution.io/v1alpha1
kind: Project
metadata:
  name: Projeto Teste
  description: Projeto para testar férias com layoff
spec:
  name: Projeto Teste
  description: Projeto para testar férias com layoff
  status: InProgress
  vacationRules:
    maxConcurrentVacations: 2
    allowLayoffVacations: true
    requireLayoffVacationPeriod: false
    layoffPeriods:
      - startDate: "2024-06-01"
        endDate: "2024-06-30"
      - startDate: "2024-12-15"
        endDate: "2024-12-31"
EOF

echo "3. Criando recursos..."
mkdir -p resources
cargo run --quiet -- create resource "João Silva" "Desenvolvedor"
cargo run --quiet -- create resource "Maria Oliveira" "Designer"
cargo run --quiet -- create resource "Pedro Santos" "QA"

echo "4. Adicionando férias para João (não coincide com layoff)..."
cargo run --quiet -- create vacation --resource "João Silva" --start-date "2024-05-01" --end-date "2024-05-15"

echo "5. Adicionando férias para Maria (coincide com layoff)..."
cargo run --quiet -- create vacation --resource "Maria Oliveira" --start-date "2024-06-01" --end-date "2024-06-15"

echo "6. Adicionando férias para Pedro (parcialmente coincide com layoff)..."
cargo run --quiet -- create vacation --resource "Pedro Santos" --start-date "2024-12-10" --end-date "2024-12-20"

echo "7. Gerando relatório de férias..."
cargo run --quiet -- report vacation

echo "8. Conteúdo do relatório:"
cat vacation_report.csv

echo "9. Validando férias..."
cargo run --quiet -- validate vacations 