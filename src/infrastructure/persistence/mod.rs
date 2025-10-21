pub mod company_repository;
pub mod config_repository;
pub mod manifests;
pub mod project_repository;
pub mod resource_repository;
pub mod task_repository;

// Financial repositories
pub mod repositories {
    pub mod budget_repository;
    pub mod cost_repository;

    pub use budget_repository::BudgetRepository;
    pub use cost_repository::CostRepository;
}
