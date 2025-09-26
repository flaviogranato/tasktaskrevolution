#!/bin/bash

# Script para corrigir todos os usos de Resource::new e CreateResourceParams

echo "Corrigindo Resource::new e CreateResourceParams..."

# Função para adicionar scope aos CreateResourceParams
fix_create_resource_params() {
    local file="$1"
    echo "Corrigindo CreateResourceParams em $file"
    
    # Adicionar scope: ResourceScope::Company, se não existir
    sed -i 's/scope: ResourceScope::Company,$/scope: ResourceScope::Company,/g' "$file"
    sed -i '/scope: ResourceScope::Company,$/!s/end_date: None,$/end_date: None,\n            scope: ResourceScope::Company,/g' "$file"
}

# Função para corrigir Resource::new
fix_resource_new() {
    local file="$1"
    echo "Corrigindo Resource::new em $file"
    
    # Adicionar ResourceScope::Company e None para vacations
    sed -i 's/Resource::new(\([^,]*\),\([^,]*\),\([^,]*\),\([^,]*\),\([^,]*\),\([^,]*\),\([^,]*\),\([^,]*\))/Resource::new(\1,\2,\3,\4,ResourceScope::Company,None,\6,\7,None,\8)/g' "$file"
}

# Função para corrigir Resource::<Available>::new
fix_resource_available_new() {
    local file="$1"
    echo "Corrigindo Resource::<Available>::new em $file"
    
    # Adicionar ResourceScope::Company e None para vacations
    sed -i 's/Resource::<Available>::new(\([^,]*\),\([^,]*\),\([^,]*\),\([^,]*\),\([^,]*\),\([^,]*\),\([^,]*\),\([^,]*\))/Resource::<Available>::new(\1,\2,\3,\4,ResourceScope::Company,None,\6,\7,None,\8)/g' "$file"
}

# Função para corrigir Resource::<Available> { ... }
fix_resource_struct() {
    local file="$1"
    echo "Corrigindo Resource::<Available> struct em $file"
    
    # Adicionar project_id: None, e scope: ResourceScope::Company,
    sed -i 's/Resource::<Available> {/Resource::<Available> {\n            project_id: None,\n            scope: ResourceScope::Company,/g' "$file"
}

# Função para corrigir ResourceSpec
fix_resource_spec() {
    local file="$1"
    echo "Corrigindo ResourceSpec em $file"
    
    # Adicionar project_id: None, e scope: ResourceScope::Company,
    sed -i 's/spec: crate::infrastructure::persistence::manifests::resource_manifest::ResourceSpec {/spec: crate::infrastructure::persistence::manifests::resource_manifest::ResourceSpec {\n                project_id: None,\n                scope: ResourceScope::Company,/g' "$file"
}

# Lista de arquivos para corrigir
files=(
    "src/application/create/resource.rs"
    "src/application/create/vacation.rs"
    "src/application/list/resources.rs"
    "src/application/task/assign_resource.rs"
    "src/application/report/wip.rs"
    "src/application/project/assign_resource_to_task.rs"
    "src/application/template/create_from_template.rs"
    "src/domain/project_management/resource_allocation.rs"
    "src/domain/resource_management/resource.rs"
    "src/infrastructure/persistence/resource_repository.rs"
    "src/interface/cli/commands/wip.rs"
    "src/interface/cli/handlers/resource_handler.rs"
    "src/interface/cli/simplified_executor.rs"
)

# Aplicar correções
for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        echo "Processando $file..."
        
        # Aplicar todas as correções
        fix_create_resource_params "$file"
        fix_resource_new "$file"
        fix_resource_available_new "$file"
        fix_resource_struct "$file"
        fix_resource_spec "$file"
        
        echo "Concluído: $file"
    else
        echo "Arquivo não encontrado: $file"
    fi
done

echo "Correções aplicadas! Testando compilação..."
