#!/usr/bin/env python3

import re
import sys

files = [
    "src/application/report/vacation.rs",
    "src/application/report/wip.rs", 
    "src/application/resource/update_resource.rs",
    "src/interface/cli/commands/wip.rs"
]

for file_path in files:
    print(f"Processing {file_path}")
    
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Find the find_by_code method and add find_by_company after it
    pattern = r'(fn find_by_code\(&self, code: &str\) -> Result<Option<AnyResource>, AppError> \{[^}]*\})'
    
    def replace_func(match):
        original = match.group(1)
        addition = """
        fn find_by_company(&self, _company_code: &str) -> Result<Vec<AnyResource>, AppError> {
            Ok(vec![])
        }"""
        return original + addition
    
    new_content = re.sub(pattern, replace_func, content, flags=re.DOTALL)
    
    if new_content != content:
        with open(file_path, 'w') as f:
            f.write(new_content)
        print(f"  Updated {file_path}")
    else:
        print(f"  No changes needed for {file_path}")

print("Done!")
