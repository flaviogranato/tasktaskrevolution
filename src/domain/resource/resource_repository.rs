use crate::domain::shared_kernel::errors::DomainError;

use super::resource::Resource;

pub trait ResourceRepository {
    fn save(&self, resource: Resource) -> Result<Resource, DomainError>;
    fn find_all(&self) -> Result<Vec<Resource>, DomainError>;
}
