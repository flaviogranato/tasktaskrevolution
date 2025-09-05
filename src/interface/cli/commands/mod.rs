use clap::Subcommand;
use std::path::PathBuf;

pub mod company;
pub mod project;
pub mod task;
pub mod resource;
pub mod list;
pub mod update;
pub mod delete;
pub mod link;
pub mod unlink;
pub mod report;
pub mod validate;
pub mod template;

pub use company::CompanyCommand;
pub use project::ProjectCommand;
pub use task::TaskCommand;
pub use resource::ResourceCommand;
pub use list::ListCommand;
pub use update::UpdateCommand;
pub use delete::DeleteCommand;
pub use link::LinkCommand;
pub use unlink::UnlinkCommand;
pub use report::ReportCommand;
pub use validate::ValidateCommand;
pub use template::TemplateCommand;
