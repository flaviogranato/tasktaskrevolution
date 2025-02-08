use crate::domain::shared_kernel::errors::DomainError;

use super::resource::Resource;

pub trait ResourceRepository {
    fn save(&self, resource: Resource) -> Result<Resource, DomainError>;
}
