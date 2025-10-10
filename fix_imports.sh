#!/bin/bash

# Script para corrigir imports em arquivos Rust
# Adiciona imports necessários para DomainError e DomainResult

files=(
    "src/application/resource/realtime_conflict_monitor.rs"
    "src/application/validate/data_integrity.rs"
)

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        echo "Fixing $file"
        
        # Adicionar imports necessários nos módulos de teste
        sed -i '/#\[cfg(test)\]/a\    use crate::domain::shared::errors::{DomainError, DomainResult};' "$file"
        
        # Adicionar imports para traits específicos
        if [[ "$file" == *"realtime_conflict_monitor.rs" ]]; then
            sed -i '/use crate::domain::shared::errors::{DomainError, DomainResult};/a\    use crate::domain::project_management::AnyProject;' "$file"
            sed -i '/use crate::domain::project_management::AnyProject;/a\    use crate::domain::project_management::repository::ProjectRepositoryWithId;' "$file"
            sed -i '/use crate::domain::project_management::repository::ProjectRepositoryWithId;/a\    use crate::domain::resource_management::repository::ResourceRepositoryWithId;' "$file"
        fi
        
        if [[ "$file" == *"data_integrity.rs" ]]; then
            sed -i '/use crate::domain::shared::errors::{DomainError, DomainResult};/a\    use crate::domain::company_management::Company;' "$file"
        fi
    fi
done

echo "Done fixing imports"

