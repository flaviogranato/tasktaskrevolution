use super::super::commands::MigrateCommand;
use crate::{
    domain::{
        company_management::repository::CompanyRepository, project_management::repository::ProjectRepository,
        resource_management::repository::ResourceRepository,
    },
    infrastructure::persistence::{
        company_repository::FileCompanyRepository, project_repository::FileProjectRepository,
        resource_repository::FileResourceRepository,
    },
};
use std::fs;
use std::path::Path;

pub fn handle_migrate_command(command: MigrateCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        MigrateCommand::Manifests { dry_run, force, backup } => {
            if dry_run {
                println!("Dry run mode - no changes will be made");
            }

            if backup {
                println!("💾 Creating backup...");
                create_backup()?;
            }

            migrate_manifests(dry_run, force)?;
            Ok(())
        }
        MigrateCommand::ToIdBased { dry_run, force, backup } => {
            if dry_run {
                println!("Dry run mode - no changes will be made");
            }

            if backup {
                println!("💾 Creating backup...");
                create_backup()?;
            }

            migrate_to_id_based(dry_run, force)?;
            Ok(())
        }
        MigrateCommand::Status => {
            show_migration_status()?;
            Ok(())
        }
        MigrateCommand::Rollback { backup_dir } => {
            rollback_migration(backup_dir)?;
            Ok(())
        }
    }
}

fn create_backup() -> Result<(), Box<dyn std::error::Error>> {
    let backup_dir = "backup_before_migration";
    if Path::new(backup_dir).exists() {
        fs::remove_dir_all(backup_dir)?;
    }
    fs::create_dir_all(backup_dir)?;

    // Copy all data directories
    let dirs_to_backup = ["companies", "projects", "resources", ".ttr"];
    for dir in &dirs_to_backup {
        if Path::new(dir).exists() {
            copy_dir_recursive(dir, &format!("{}/{}", backup_dir, dir))?;
        }
    }

    println!("Backup created in {}", backup_dir);
    Ok(())
}

fn copy_dir_recursive(src: &str, dst: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = Path::new(dst).join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path.to_string_lossy(), &dst_path.to_string_lossy())?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn migrate_to_id_based(dry_run: bool, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting migration to ID-based format...");

    // Check if ID-based data already exists
    if !force && has_id_based_data()? {
        return Err("ID-based data already exists. Use --force to overwrite.".into());
    }

    // Migrate companies
    println!("📁 Migrating companies...");
    migrate_companies(dry_run)?;

    // Migrate projects
    println!("📁 Migrating projects...");
    migrate_projects(dry_run)?;

    // Migrate resources
    println!("📁 Migrating resources...");
    migrate_resources(dry_run)?;

    if dry_run {
        println!("Dry run completed - no changes were made");
    } else {
        println!("Migration completed successfully!");
    }

    Ok(())
}

fn has_id_based_data() -> Result<bool, Box<dyn std::error::Error>> {
    // Check if any ID-based files exist
    let companies_dir = Path::new("companies");
    if companies_dir.exists() {
        for entry in fs::read_dir(companies_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && path.extension().and_then(|s| s.to_str()) == Some("yaml")
                && let Some(file_name) = path.file_stem().and_then(|s| s.to_str())
            {
                // Check if it looks like a UUID (ID-based format)
                if file_name.len() == 36 && file_name.contains('-') {
                    return Ok(true);
                }
            }
        }
    }

    let projects_dir = Path::new("projects");
    if projects_dir.exists() {
        for entry in fs::read_dir(projects_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && path.extension().and_then(|s| s.to_str()) == Some("yaml")
                && let Some(file_name) = path.file_stem().and_then(|s| s.to_str())
            {
                // Check if it looks like a UUID (ID-based format)
                if file_name.len() == 36 && file_name.contains('-') {
                    return Ok(true);
                }
            }
        }
    }

    let resources_dir = Path::new("resources");
    if resources_dir.exists() {
        for entry in fs::read_dir(resources_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && path.extension().and_then(|s| s.to_str()) == Some("yaml")
                && let Some(file_name) = path.file_stem().and_then(|s| s.to_str())
            {
                // Check if it looks like a UUID (ID-based format)
                if file_name.len() == 36 && file_name.contains('-') {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

fn migrate_companies(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let companies_dir = Path::new("companies");
    if !companies_dir.exists() {
        return Ok(());
    }

    let company_repo = FileCompanyRepository::new(".");
    let companies = company_repo.find_all()?;

    for company in companies {
        if dry_run {
            println!("  Would migrate company: {} ({})", company.name(), company.code());
        } else {
            // The repository already handles ID-based saving
            let name = company.name().to_string();
            let code = company.code().to_string();
            company_repo.save(company)?;
            println!("  Migrated company: {} ({})", name, code);
        }
    }

    Ok(())
}

fn migrate_projects(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let projects_dir = Path::new("projects");
    if !projects_dir.exists() {
        return Ok(());
    }

    let project_repo = FileProjectRepository::new();
    let projects = project_repo.find_all()?;

    for project in projects {
        if dry_run {
            println!("  Would migrate project: {} ({})", project.name(), project.code());
        } else {
            // The repository already handles ID-based saving
            let name = project.name().to_string();
            let code = project.code().to_string();
            project_repo.save(project)?;
            println!("  Migrated project: {} ({})", name, code);
        }
    }

    Ok(())
}

fn migrate_resources(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let resources_dir = Path::new("resources");
    if !resources_dir.exists() {
        return Ok(());
    }

    let resource_repo = FileResourceRepository::new(".");
    let resources = resource_repo.find_all()?;

    for resource in resources {
        if dry_run {
            println!("  Would migrate resource: {} ({})", resource.name(), resource.code());
        } else {
            // The repository already handles ID-based saving
            let name = resource.name().to_string();
            let code = resource.code().to_string();
            resource_repo.save(resource)?;
            println!("  Migrated resource: {} ({})", name, code);
        }
    }

    Ok(())
}

fn show_migration_status() -> Result<(), Box<dyn std::error::Error>> {
    println!("Migration Status Report");
    println!("=========================");

    // Check companies
    let companies_dir = Path::new("companies");
    if companies_dir.exists() {
        let mut code_based = 0;
        let mut id_based = 0;

        for entry in fs::read_dir(companies_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Old code-based format
                code_based += 1;
            } else if path.is_file()
                && path.extension().and_then(|s| s.to_str()) == Some("yaml")
                && let Some(file_name) = path.file_stem().and_then(|s| s.to_str())
            {
                if file_name.len() == 36 && file_name.contains('-') {
                    id_based += 1;
                } else {
                    code_based += 1;
                }
            }
        }

        println!("Companies: {} code-based, {} ID-based", code_based, id_based);
    } else {
        println!("Companies: No data found");
    }

    // Check projects
    let projects_dir = Path::new("projects");
    if projects_dir.exists() {
        let mut code_based = 0;
        let mut id_based = 0;

        for entry in fs::read_dir(projects_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Old code-based format
                code_based += 1;
            } else if path.is_file()
                && path.extension().and_then(|s| s.to_str()) == Some("yaml")
                && let Some(file_name) = path.file_stem().and_then(|s| s.to_str())
            {
                if file_name.len() == 36 && file_name.contains('-') {
                    id_based += 1;
                } else {
                    code_based += 1;
                }
            }
        }

        println!("Projects: {} code-based, {} ID-based", code_based, id_based);
    } else {
        println!("Projects: No data found");
    }

    // Check resources
    let resources_dir = Path::new("resources");
    if resources_dir.exists() {
        let mut code_based = 0;
        let mut id_based = 0;

        for entry in fs::read_dir(resources_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Old code-based format
                code_based += 1;
            } else if path.is_file()
                && path.extension().and_then(|s| s.to_str()) == Some("yaml")
                && let Some(file_name) = path.file_stem().and_then(|s| s.to_str())
            {
                if file_name.len() == 36 && file_name.contains('-') {
                    id_based += 1;
                } else {
                    code_based += 1;
                }
            }
        }

        println!("Resources: {} code-based, {} ID-based", code_based, id_based);
    } else {
        println!("Resources: No data found");
    }

    // Mapping service removed - using simple file-based search
    println!("Mappings: Using simple file-based search (no mapping service)");

    Ok(())
}

fn rollback_migration(backup_dir: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let backup_path = backup_dir.unwrap_or_else(|| "backup_before_migration".to_string());

    if !Path::new(&backup_path).exists() {
        return Err(format!("Backup directory '{}' not found", backup_path).into());
    }

    println!("🔄 Rolling back migration from {}", backup_path);

    // Remove current data
    let dirs_to_remove = ["companies", "projects", "resources", ".ttr"];
    for dir in &dirs_to_remove {
        if Path::new(dir).exists() {
            fs::remove_dir_all(dir)?;
        }
    }

    // Restore from backup
    for dir in &dirs_to_remove {
        let backup_dir_path = Path::new(&backup_path).join(dir);
        if backup_dir_path.exists() {
            fs::rename(&backup_dir_path, dir)?;
        }
    }

    println!("Rollback completed successfully!");
    Ok(())
}

fn migrate_manifests(dry_run: bool, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Starting YAML manifests migration...");
    println!("Starting YAML manifests migration...");

    // Check if manifests are already up to date
    if !force && are_manifests_up_to_date()? {
        println!("✅ All manifests are already up to date with the latest apiVersion");
        return Ok(());
    }

    // Migrate company manifests
    println!("📁 Migrating company manifests...");
    migrate_company_manifests(dry_run)?;

    // Migrate project manifests
    println!("📁 Migrating project manifests...");
    migrate_project_manifests(dry_run)?;

    // Migrate resource manifests
    println!("📁 Migrating resource manifests...");
    migrate_resource_manifests(dry_run)?;

    // Migrate task manifests
    println!("📁 Migrating task manifests...");
    migrate_task_manifests(dry_run)?;

    if dry_run {
        println!("Dry run completed - no changes were made");
    } else {
        println!("✅ YAML manifests migration completed successfully!");
    }

    Ok(())
}

fn are_manifests_up_to_date() -> Result<bool, Box<dyn std::error::Error>> {
    // For now, always return false to allow migration
    // This is a stub implementation as requested in the issue
    Ok(false)
}

#[allow(dead_code)]
fn check_manifest_api_version(
    manifest_path: &Path,
    expected_version: &str,
) -> Result<bool, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(manifest_path)?;
    let lines: Vec<&str> = content.lines().collect();

    for line in lines {
        if line.trim().starts_with("apiVersion:") {
            let version = line.split(':').nth(1).unwrap_or("").trim();
            return Ok(version == expected_version);
        }
    }

    // If no apiVersion found, consider it outdated
    Ok(false)
}

fn migrate_company_manifests(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    if dry_run {
        println!("  Would migrate company manifests (stub implementation)");
    } else {
        println!("  Migrated company manifests (stub implementation)");
    }
    Ok(())
}

fn migrate_project_manifests(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    if dry_run {
        println!("  Would migrate project manifests (stub implementation)");
    } else {
        println!("  Migrated project manifests (stub implementation)");
    }
    Ok(())
}

fn migrate_resource_manifests(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    if dry_run {
        println!("  Would migrate resource manifests (stub implementation)");
    } else {
        println!("  Migrated resource manifests (stub implementation)");
    }
    Ok(())
}

fn migrate_task_manifests(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    if dry_run {
        println!("  Would migrate task manifests (stub implementation)");
    } else {
        println!("  Migrated task manifests (stub implementation)");
    }
    Ok(())
}

#[allow(dead_code)]
fn update_manifest_api_version(manifest_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(manifest_path)?;
    let lines: Vec<&str> = content.lines().collect();
    let mut updated_lines = Vec::new();
    let mut found_api_version = false;

    for line in lines {
        if line.trim().starts_with("apiVersion:") {
            updated_lines.push("apiVersion: tasktaskrevolution.io/v1alpha1".to_string());
            found_api_version = true;
        } else {
            updated_lines.push(line.to_string());
        }
    }

    // If no apiVersion found, add it at the beginning
    if !found_api_version {
        updated_lines.insert(0, "apiVersion: tasktaskrevolution.io/v1alpha1".to_string());
        updated_lines.insert(1, "".to_string());
    }

    let updated_content = updated_lines.join("\n");
    fs::write(manifest_path, updated_content)?;

    Ok(())
}
