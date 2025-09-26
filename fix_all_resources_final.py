#!/usr/bin/env python3

import os
import re

def fix_resource_new_final(file_path):
    """Corrige todos os usos de Resource::new para incluir os novos parâmetros"""
    
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Padrão para Resource::new com 8 parâmetros (multilinha)
    pattern = r'Resource::new\(\s*([^,]+),\s*([^,]+),\s*([^,]+),\s*([^,]+),\s*([^,]+),\s*([^,]+),\s*([^,]+),\s*([^,]+)\s*\)'
    replacement = r'Resource::new(\1, \2, \3, \4, ResourceScope::Company, None, \6, \7, None, \8)'
    content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
    
    # Padrão para Resource::<Available>::new com 8 parâmetros (multilinha)
    pattern = r'Resource::<Available>::new\(\s*([^,]+),\s*([^,]+),\s*([^,]+),\s*([^,]+),\s*([^,]+),\s*([^,]+),\s*([^,]+),\s*([^,]+)\s*\)'
    replacement = r'Resource::<Available>::new(\1, \2, \3, \4, ResourceScope::Company, None, \6, \7, None, \8)'
    content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
    
    # Padrão para Resource::<Available> { ... } - adicionar campos faltantes
    pattern = r'Resource::<Available>\s*\{'
    replacement = r'Resource::<Available> {\n            project_id: None,\n            scope: ResourceScope::Company,'
    content = re.sub(pattern, replacement, content)
    
    with open(file_path, 'w') as f:
        f.write(content)
    
    print(f"Corrigido: {file_path}")

def main():
    # Lista de arquivos para corrigir
    files = [
        "src/application/list/resources.rs",
        "src/application/project/assign_resource_to_task.rs",
        "src/application/task/assign_resource.rs",
        "src/domain/resource_management/resource.rs",
        "src/infrastructure/persistence/manifests/resource_manifest.rs",
        "src/infrastructure/persistence/resource_repository.rs",
        "src/interface/cli/commands/wip.rs"
    ]
    
    for file_path in files:
        if os.path.exists(file_path):
            fix_resource_new_final(file_path)
        else:
            print(f"Arquivo não encontrado: {file_path}")

if __name__ == "__main__":
    main()
