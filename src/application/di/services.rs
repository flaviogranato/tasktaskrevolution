use crate::{
    application::{
        build_use_case::BuildUseCase,
        company_management::CreateCompanyUseCase,
        create::{
            project::CreateProjectUseCase,
            resource::CreateResourceUseCase,
            task::CreateTaskUseCase,
            time_off::CreateTimeOffUseCase,
            vacation::CreateVacationUseCase,
        },
        init::InitManagerUseCase,
        list::{
            projects::ListProjectsUseCase,
            resources::ListResourcesUseCase,
            tasks::ListTasksUseCase,
        },
        project::{
            assign_resource_to_task::AssignResourceToTaskUseCase,
            cancel_project::CancelProjectUseCase,
            describe_project::DescribeProjectUseCase,
            update_project::UpdateProjectUseCase,
        },
        report::{
            task::TaskReportUseCase,
            vacation::VacationReportUseCase,
        },
        resource::{
            deactivate_resource::DeactivateResourceUseCase,
            describe_resource::DescribeResourceUseCase,
            update_resource::UpdateResourceUseCase,
        },
        task::{
            delete_task::DeleteTaskUseCase,
            describe_task::DescribeTaskUseCase,
            link_task::LinkTaskUseCase,
            update_task::UpdateTaskUseCase,
        },
        template::{
            create_from_template::CreateFromTemplateUseCase,
            list_templates::ListTemplatesUseCase,
            load_template::LoadTemplateUseCase,
        },
        validate::{
            business_rules::ValidateBusinessRulesUseCase,
            data_integrity::ValidateDataIntegrityUseCase,
            entities::ValidateEntitiesUseCase,
            system::ValidateSystemUseCase,
        },
    },
    infrastructure::persistence::{
        company_repository::FileCompanyRepository,
        config_repository::FileConfigRepository,
        project_repository::FileProjectRepository,
        resource_repository::FileResourceRepository,
        task_repository::FileTaskRepository,
    },
};
use super::traits::Injectable;
use std::any::Any;
use std::sync::Arc;

/// Serviço de repositórios
pub struct RepositoryService {
    pub company_repository: FileCompanyRepository,
    pub config_repository: FileConfigRepository,
    pub project_repository: FileProjectRepository,
    pub resource_repository: FileResourceRepository,
    pub task_repository: FileTaskRepository,
}

impl RepositoryService {
    pub fn new() -> Self {
        Self {
            company_repository: FileCompanyRepository::new("."),
            config_repository: FileConfigRepository::new(),
            project_repository: FileProjectRepository::new(),
            resource_repository: FileResourceRepository::new("."),
            task_repository: FileTaskRepository::new("."),
        }
    }
}

impl Injectable for RepositoryService {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Serviço de casos de uso de criação
pub struct CreateUseCaseService {
    pub create_company: CreateCompanyUseCase,
    pub create_project: CreateProjectUseCase,
    pub create_resource: CreateResourceUseCase,
    pub create_task: CreateTaskUseCase,
    pub create_time_off: CreateTimeOffUseCase,
    pub create_vacation: CreateVacationUseCase,
}

impl CreateUseCaseService {
    pub fn new(repos: &RepositoryService) -> Self {
        Self {
            create_company: CreateCompanyUseCase::new(repos.company_repository.clone()),
            create_project: CreateProjectUseCase::new(
                repos.project_repository.clone(),
                repos.company_repository.clone(),
            ),
            create_resource: CreateResourceUseCase::new(repos.resource_repository.clone()),
            create_task: CreateTaskUseCase::new(
                repos.task_repository.clone(),
                repos.project_repository.clone(),
            ),
            create_time_off: CreateTimeOffUseCase::new(repos.resource_repository.clone()),
            create_vacation: CreateVacationUseCase::new(repos.resource_repository.clone()),
        }
    }
}

impl Injectable for CreateUseCaseService {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Serviço de casos de uso de listagem
pub struct ListUseCaseService {
    pub list_projects: ListProjectsUseCase,
    pub list_resources: ListResourcesUseCase,
    pub list_tasks: ListTasksUseCase,
}

impl ListUseCaseService {
    pub fn new(repos: &RepositoryService) -> Self {
        Self {
            list_projects: ListProjectsUseCase::new(repos.project_repository.clone()),
            list_resources: ListResourcesUseCase::new(repos.resource_repository.clone()),
            list_tasks: ListTasksUseCase::new(repos.task_repository.clone()),
        }
    }
}

impl Injectable for ListUseCaseService {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Serviço de casos de uso de projeto
pub struct ProjectUseCaseService {
    pub assign_resource: AssignResourceToTaskUseCase,
    pub cancel_project: CancelProjectUseCase,
    pub describe_project: DescribeProjectUseCase,
    pub update_project: UpdateProjectUseCase,
}

impl ProjectUseCaseService {
    pub fn new(repos: &RepositoryService) -> Self {
        Self {
            assign_resource: AssignResourceToTaskUseCase::new(repos.project_repository.clone()),
            cancel_project: CancelProjectUseCase::new(repos.project_repository.clone()),
            describe_project: DescribeProjectUseCase::new(repos.project_repository.clone()),
            update_project: UpdateProjectUseCase::new(repos.project_repository.clone()),
        }
    }
}

impl Injectable for ProjectUseCaseService {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Serviço de casos de uso de tarefa
pub struct TaskUseCaseService {
    pub delete_task: DeleteTaskUseCase,
    pub describe_task: DescribeTaskUseCase,
    pub link_task: LinkTaskUseCase,
    pub update_task: UpdateTaskUseCase,
}

impl TaskUseCaseService {
    pub fn new(repos: &RepositoryService) -> Self {
        Self {
            delete_task: DeleteTaskUseCase::new(repos.task_repository.clone()),
            describe_task: DescribeTaskUseCase::new(repos.task_repository.clone()),
            link_task: LinkTaskUseCase::new(repos.project_repository.clone()),
            update_task: UpdateTaskUseCase::new(repos.task_repository.clone()),
        }
    }
}

impl Injectable for TaskUseCaseService {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Serviço de casos de uso de recurso
pub struct ResourceUseCaseService {
    pub deactivate_resource: DeactivateResourceUseCase,
    pub describe_resource: DescribeResourceUseCase,
    pub update_resource: UpdateResourceUseCase,
}

impl ResourceUseCaseService {
    pub fn new(repos: &RepositoryService) -> Self {
        Self {
            deactivate_resource: DeactivateResourceUseCase::new(repos.resource_repository.clone()),
            describe_resource: DescribeResourceUseCase::new(repos.resource_repository.clone()),
            update_resource: UpdateResourceUseCase::new(repos.resource_repository.clone()),
        }
    }
}

impl Injectable for ResourceUseCaseService {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Serviço de casos de uso de template
pub struct TemplateUseCaseService {
    pub create_from_template: CreateFromTemplateUseCase,
    pub list_templates: ListTemplatesUseCase,
    pub load_template: LoadTemplateUseCase,
}

impl TemplateUseCaseService {
    pub fn new(repos: &RepositoryService) -> Self {
        Self {
            create_from_template: CreateFromTemplateUseCase::new(
                CreateProjectUseCase::new(
                    repos.project_repository.clone(),
                    repos.company_repository.clone(),
                ),
                CreateResourceUseCase::new(repos.resource_repository.clone()),
                CreateTaskUseCase::new(
                    repos.task_repository.clone(),
                    repos.project_repository.clone(),
                ),
            ),
            list_templates: ListTemplatesUseCase::new(),
            load_template: LoadTemplateUseCase::new(),
        }
    }
}

impl Injectable for TemplateUseCaseService {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Serviço de casos de uso de validação
pub struct ValidationUseCaseService<'a> {
    pub validate_business_rules: ValidateBusinessRulesUseCase<'a>,
    pub validate_data_integrity: ValidateDataIntegrityUseCase<'a>,
    pub validate_entities: ValidateEntitiesUseCase<'a>,
    pub validate_system: ValidateSystemUseCase<'a>,
}

impl<'a> ValidationUseCaseService<'a> {
    pub fn new(repos: &'a RepositoryService) -> Self {
        Self {
            validate_business_rules: ValidateBusinessRulesUseCase::new(
                &repos.project_repository,
                &repos.resource_repository,
                &repos.company_repository,
            ),
            validate_data_integrity: ValidateDataIntegrityUseCase::new(
                &repos.project_repository,
                &repos.resource_repository,
                &repos.company_repository,
            ),
            validate_entities: ValidateEntitiesUseCase::new(
                &repos.project_repository,
                &repos.resource_repository,
                &repos.company_repository,
            ),
            validate_system: ValidateSystemUseCase::new(
                repos.project_repository.clone(),
                repos.resource_repository.clone(),
                repos.company_repository.clone(),
            ),
        }
    }
}

impl Injectable for ValidationUseCaseService<'_> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Serviço de casos de uso de relatórios
pub struct ReportUseCaseService {
    pub task_report: TaskReportUseCase,
    pub vacation_report: VacationReportUseCase,
}

impl ReportUseCaseService {
    pub fn new(repos: &RepositoryService) -> Self {
        Self {
            task_report: TaskReportUseCase::new(repos.project_repository.clone()),
            vacation_report: VacationReportUseCase::new(
                repos.project_repository.clone(),
                repos.resource_repository.clone(),
            ),
        }
    }
}

impl Injectable for ReportUseCaseService {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Serviço de inicialização
pub struct InitService {
    pub init_manager: InitManagerUseCase,
}

impl InitService {
    pub fn new(repos: &RepositoryService) -> Self {
        Self {
            init_manager: InitManagerUseCase::new(repos.config_repository.clone()),
        }
    }
}

impl Injectable for InitService {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
