#!/bin/bash

# Fix src/application/resource/deactivate_resource.rs
sed -i '/fn find_by_code.*Ok(None)/a\
\
        fn save_in_hierarchy(&self, resource: AnyResource, _company_code: &str, _project_code: Option<&str>) -> Result<AnyResource, DomainError> {\
            self.save(resource)\
        }' src/application/resource/deactivate_resource.rs

# Fix src/application/resource/describe_resource.rs
sed -i '/fn find_by_code.*Ok(None)/a\
\
        fn save_in_hierarchy(&self, resource: AnyResource, _company_code: &str, _project_code: Option<&str>) -> Result<AnyResource, DomainError> {\
            self.save(resource)\
        }' src/application/resource/describe_resource.rs

# Fix src/application/resource/update_resource.rs
sed -i '/fn find_by_code.*Ok(None)/a\
\
        fn save_in_hierarchy(&self, resource: AnyResource, _company_code: &str, _project_code: Option<&str>) -> Result<AnyResource, DomainError> {\
            self.save(resource)\
        }' src/application/resource/update_resource.rs

# Fix src/application/task/assign_resource.rs (ResourceRepository mock)
sed -i '/fn find_by_code.*Ok(self.resources.borrow().get(code).cloned())/a\
\
        fn save_in_hierarchy(&self, resource: AnyResource, _company_code: &str, _project_code: Option<&str>) -> Result<AnyResource, DomainError> {\
            self.save(resource)\
        }' src/application/task/assign_resource.rs

echo "Fixed all mocks"
