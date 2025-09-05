use super::{DIContainer, ServiceRegistrar, RepositoryService, CreateUseCaseService, ListUseCaseService, ProjectUseCaseService, TaskUseCaseService, ResourceUseCaseService, TemplateUseCaseService, ValidationUseCaseService, ReportUseCaseService, InitService};

/// Factory para configurar o container de DI com todos os serviços
pub struct DIFactory;

impl DIFactory {
    /// Cria um container configurado com todos os serviços
    pub fn create_container() -> Result<DIContainer, String> {
        let mut container = DIContainer::new();
        
        // Registra repositórios como singletons
        let repos = RepositoryService::new();
        container.register_instance(repos)?;
        
        // Registra serviços de casos de uso
        Self::register_use_case_services(&mut container)?;
        
        Ok(container)
    }
    
    fn register_use_case_services(container: &mut DIContainer) -> Result<(), String> {
        // Resolve repositórios para criar os serviços
        let repos: std::sync::Arc<RepositoryService> = container.resolve()?;
        
        // Registra serviços de criação
        let create_service = CreateUseCaseService::new(&repos);
        container.register_instance(create_service)?;
        
        // Registra serviços de listagem
        let list_service = ListUseCaseService::new(&repos);
        container.register_instance(list_service)?;
        
        // Registra serviços de projeto
        let project_service = ProjectUseCaseService::new(&repos);
        container.register_instance(project_service)?;
        
        // Registra serviços de tarefa
        let task_service = TaskUseCaseService::new(&repos);
        container.register_instance(task_service)?;
        
        // Registra serviços de recurso
        let resource_service = ResourceUseCaseService::new(&repos);
        container.register_instance(resource_service)?;
        
        // Registra serviços de template
        let template_service = TemplateUseCaseService::new(&repos);
        container.register_instance(template_service)?;
        
        // Registra serviços de validação
        let validation_service = ValidationUseCaseService::new(&repos);
        container.register_instance(validation_service)?;
        
        // Registra serviços de relatório
        let report_service = ReportUseCaseService::new(&repos);
        container.register_instance(report_service)?;
        
        // Registra serviços de inicialização
        let init_service = InitService::new(&repos);
        container.register_instance(init_service)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_container() {
        let container = DIFactory::create_container();
        assert!(container.is_ok());
    }
    
    #[test]
    fn test_resolve_repository_service() {
        let container = DIFactory::create_container().unwrap();
        let repos: std::sync::Arc<RepositoryService> = container.resolve().unwrap();
        assert!(repos.company_repository.base_path().exists() || repos.company_repository.base_path().to_string_lossy() == ".");
    }
    
    #[test]
    fn test_resolve_create_service() {
        let container = DIFactory::create_container().unwrap();
        let create_service: std::sync::Arc<CreateUseCaseService> = container.resolve().unwrap();
        // Verifica se o serviço foi criado corretamente
        assert!(true); // Se chegou até aqui, o serviço foi resolvido
    }
}
