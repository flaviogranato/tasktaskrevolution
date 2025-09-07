pub mod company;
pub mod project;
pub mod resource;
pub mod task;

pub use company::*;
pub use project::*;
pub use resource::*;
pub use task::*;

/// Trait base para todos os Commands (operações de escrita)
pub trait Command {
    type Result;
}

/// Trait base para todos os Queries (operações de leitura)
pub trait Query {
    type Result;
}
