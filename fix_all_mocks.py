#!/usr/bin/env python3
import re
import os

files_to_fix = [
    "src/application/project/assign_resource_to_task.rs", 
    "src/application/report/vacation.rs",
    "src/application/report/wip.rs",
    "src/application/resource/describe_resource.rs",
    "src/application/resource/update_resource.rs",
    "src/application/task/assign_resource.rs",
    "src/interface/cli/commands/wip.rs"
]

for file_path in files_to_fix:
    if not os.path.exists(file_path):
        continue
        
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Fix the broken pattern - more specific regex
    # Look for the pattern where find_all_with_context is incorrectly placed
    pattern = r'fn find_by_company\(&self, _company_code: &str\) -> Result<Vec<AnyResource>, AppError> \{\s*fn find_all_with_context\(&self\) -> Result<Vec<\(AnyResource, String, Vec<String>\), AppError> \{\s*Ok\(vec!\[\]\)\s*\}\s*Ok\(vec!\[\]\)\s*\}'
    
    replacement = '''fn find_by_company(&self, _company_code: &str) -> Result<Vec<AnyResource>, AppError> {
            Ok(vec![])
        }
        fn find_all_with_context(&self) -> Result<Vec<(AnyResource, String, Vec<String>)>, AppError> {
            Ok(vec![])
        }'''
    
    # Apply the fix
    new_content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
    
    if new_content != content:
        with open(file_path, 'w') as f:
            f.write(new_content)
        print(f"Fixed {file_path}")
    else:
        print(f"No changes needed for {file_path}")

print("All files processed!")
