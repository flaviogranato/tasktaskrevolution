use clap::Subcommand;

#[derive(Subcommand)]
pub enum ValidateCommand {
    /// Validate business rules
    BusinessRules,
    /// Validate data integrity
    DataIntegrity,
    /// Validate entities
    Entities,
    /// Validate system
    System,
}
