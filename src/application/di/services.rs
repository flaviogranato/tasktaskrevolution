use crate::{
    application::{
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
            project_repository: FileProjectRepository::with_base_path(".".into()),
            resource_repository: FileResourceRepository::new("."),
            task_repository: FileTaskRepository::new("."),
        }
    }
}

impl Default for RepositoryService {
    fn default() -> Self {
        Self::new()
    }
}

impl Injectable for RepositoryService {
    fn as_any(&self) -> &dyn Any {
        self
    }
}


/// Serviço de casos de uso de criação
pub struct CreateUseCaseService {
    pub create_company: CreateCompanyUseCase<FileCompanyRepository>,
    pub create_project: CreateProjectUseCase<FileProjectRepository>,
    pub create_resource: CreateResourceUseCase<FileResourceRepository>,
    pub create_task: CreateTaskUseCase<FileProjectRepository>,
    pub create_time_off: CreateTimeOffUseCase<FileResourceRepository>,
    pub create_vacation: CreateVacationUseCase<FileResourceRepository>,
}

impl CreateUseCaseService {
    pub fn new(repos: &RepositoryService) -> Self {
        Self {
            create_company: CreateCompanyUseCase::new(repos.company_repository.clone()),
            create_project: CreateProjectUseCase::new(repos.project_repository.clone()),
            create_resource: CreateResourceUseCase::new(repos.resource_repository.clone()),
            create_task: CreateTaskUseCase::new(repos.project_repository.clone()),
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
    pub list_projects: ListProjectsUseCase<FileProjectRepository>,
    pub list_resources: ListResourcesUseCase<FileResourceRepository>,
    pub list_tasks: ListTasksUseCase<FileProjectRepository>,
}

impl ListUseCaseService {
    pub fn new(repos: &RepositoryService) -> Self {
        Self {
            list_projects: ListProjectsUseCase::new(repos.project_repository.clone()),
            list_resources: ListResourcesUseCase::new(repos.resource_repository.clone()),
            list_tasks: ListTasksUseCase::new(repos.project_repository.clone()),
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
    pub assign_resource: AssignResourceToTaskUseCase<FileProjectRepository, FileResourceRepository>,
    pub cancel_project: CancelProjectUseCase<FileProjectRepository>,
    pub describe_project: DescribeProjectUseCase<FileProjectRepository>,
    pub update_project: UpdateProjectUseCase<FileProjectRepository>,
}

impl ProjectUseCaseService {
    pub fn new(repos: &RepositoryService) -> Self {
        Self {
            assign_resource: AssignResourceToTaskUseCase::new(
                repos.project_repository.clone(),
                repos.resource_repository.clone(),
            ),
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
    pub delete_task: DeleteTaskUseCase<FileProjectRepository>,
    pub describe_task: DescribeTaskUseCase<FileProjectRepository>,
    pub link_task: LinkTaskUseCase<FileProjectRepository>,
    pub update_task: UpdateTaskUseCase<FileProjectRepository>,
}

impl TaskUseCaseService {
    pub fn new(repos: &RepositoryService) -> Self {
        Self {
            delete_task: DeleteTaskUseCase::new(repos.project_repository.clone()),
            describe_task: DescribeTaskUseCase::new(repos.project_repository.clone()),
            link_task: LinkTaskUseCase::new(repos.project_repository.clone()),
            update_task: UpdateTaskUseCase::new(repos.project_repository.clone()),
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
    pub deactivate_resource: DeactivateResourceUseCase<FileResourceRepository>,
    pub describe_resource: DescribeResourceUseCase<FileResourceRepository>,
    pub update_resource: UpdateResourceUseCase<FileResourceRepository>,
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
    pub create_from_template: CreateFromTemplateUseCase<FileProjectRepository, FileResourceRepository>,
    pub list_templates: ListTemplatesUseCase,
    pub load_template: LoadTemplateUseCase,
}

impl TemplateUseCaseService {
    pub fn new(repos: &RepositoryService) -> Self {
        Self {
            create_from_template: CreateFromTemplateUseCase::new(
                CreateProjectUseCase::new(repos.project_repository.clone()),
                CreateResourceUseCase::new(repos.resource_repository.clone()),
                CreateTaskUseCase::new(repos.project_repository.clone()),
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
/// Nota: Por enquanto não usa ValidationUseCases devido a problemas de lifetime
pub struct ValidationUseCaseService {
    // Placeholder - ValidationUseCases serão implementados posteriormente
    _placeholder: (),
}

impl ValidationUseCaseService {
    pub fn new(_repos: &RepositoryService) -> Self {
        Self {
            _placeholder: (),
        }
    }
}

impl Injectable for ValidationUseCaseService {
    fn as_any(&self) -> &dyn Any {
        self
    }
}


/// Serviço de casos de uso de relatórios
pub struct ReportUseCaseService {
    pub task_report: TaskReportUseCase<FileProjectRepository>,
    pub vacation_report: VacationReportUseCase<FileProjectRepository, FileResourceRepository>,
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
/// Nota: Por enquanto não usa InitManagerUseCase devido a problemas de thread safety
pub struct InitService {
    // Placeholder - InitManagerUseCase será implementado posteriormente
    _placeholder: (),
}

impl InitService {
    pub fn new(_repos: &RepositoryService) -> Self {
        Self {
            _placeholder: (),
        }
    }
}

impl Injectable for InitService {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
