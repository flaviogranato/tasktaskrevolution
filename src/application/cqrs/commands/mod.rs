pub mod company;
pub mod project;
pub mod task;
pub mod resource;

pub use company::*;
pub use project::*;
pub use task::*;
pub use resource::*;

/// Trait base para todos os Commands (operações de escrita)
pub trait Command {
    type Result;
}

/// Trait base para todos os Queries (operações de leitura)
pub trait Query {
    type Result;
}
