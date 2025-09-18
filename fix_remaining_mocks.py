#!/usr/bin/env python3
import re

files_to_fix = [
    "src/application/report/wip.rs",
    "src/application/resource/describe_resource.rs", 
    "src/application/resource/update_resource.rs",
    "src/application/task/assign_resource.rs",
    "src/interface/cli/commands/wip.rs"
]

for file_path in files_to_fix:
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Fix the broken pattern
    pattern = r'fn find_by_company\(&self, _company_code: &str\) -> Result<Vec<AnyResource>, AppError> \{\s*fn find_all_with_context\(&self\) -> Result<Vec<\(AnyResource, String, Vec<String>\), AppError> \{\s*Ok\(vec!\[\]\)\s*\}\s*Ok\(vec!\[\]\)\s*\}'
    
    replacement = '''fn find_by_company(&self, _company_code: &str) -> Result<Vec<AnyResource>, AppError> {
            Ok(vec![])
        }
        fn find_all_with_context(&self) -> Result<Vec<(AnyResource, String, Vec<String>)>, AppError> {
            Ok(vec![])
        }'''
    
    new_content = re.sub(pattern, replacement, content, flags=re.MULTILINE | re.DOTALL)
    
    if new_content != content:
        with open(file_path, 'w') as f:
            f.write(new_content)
        print(f"Fixed {file_path}")
    else:
        print(f"No changes needed for {file_path}")

print("All files processed!")
