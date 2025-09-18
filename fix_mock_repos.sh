#!/bin/bash

# Script to add find_by_company method to MockResourceRepository implementations

files=(
    "src/application/report/vacation.rs"
    "src/application/report/wip.rs"
    "src/application/resource/deactivate_resource.rs"
    "src/application/resource/describe_resource.rs"
    "src/application/resource/update_resource.rs"
    "src/interface/cli/commands/wip.rs"
)

for file in "${files[@]}"; do
    echo "Processing $file"
    
    # Find the line with find_by_code method and add find_by_company after it
    sed -i '/fn find_by_code.*AnyResource.*AppError/a\        fn find_by_company(&self, _company_code: &str) -> Result<Vec<AnyResource>, AppError> {\n            Ok(vec![])\n        }' "$file"
done

echo "Done!"
