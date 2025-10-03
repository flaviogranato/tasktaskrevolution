use clap::Parser;

/// Migration commands for converting data between formats
#[derive(Parser, Debug)]
pub enum MigrateCommand {
    /// Migrate YAML manifests to new apiVersion format
    Manifests {
        /// Dry run - show what would be migrated without making changes
        #[arg(long)]
        dry_run: bool,

        /// Force migration even if manifests are already up to date
        #[arg(long)]
        force: bool,

        /// Backup existing manifests before migration
        #[arg(long)]
        backup: bool,
    },
    /// Migrate all data from code-based to ID-based format
    ToIdBased {
        /// Dry run - show what would be migrated without making changes
        #[arg(long)]
        dry_run: bool,

        /// Force migration even if ID-based data already exists
        #[arg(long)]
        force: bool,

        /// Backup existing data before migration
        #[arg(long)]
        backup: bool,
    },
    /// Show migration status
    Status,
    /// Rollback migration (restore from backup)
    Rollback {
        /// Backup directory to restore from
        #[arg(long)]
        backup_dir: Option<String>,
    },
}
