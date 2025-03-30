#[allow(non_snake_case)]
#[allow(clippy::module_inception)]
pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod interface;

pub use application::task::TaskService;
pub use domain::task::Task; 