use std::fs;
use std::path::{Path, PathBuf};

fn collect_rs_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_rs_files(&path));
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
    files
}

fn read_to_string(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_default()
}

#[test]
#[ignore] // Unignore após resolver a issue: DDD: Remover dependência de domain -> application (AppError leak)
fn domain_must_not_depend_on_application() {
    let domain_dir = Path::new("src/domain");
    for file in collect_rs_files(domain_dir) {
        let content = read_to_string(&file);
        assert!(
            !content.contains("crate::application") && !content.contains("application::"),
            "Domain file {:?} must not depend on application layer",
            file
        );
    }
}

#[test]
fn domain_must_not_depend_on_interface_or_infrastructure() {
    let domain_dir = Path::new("src/domain");
    for file in collect_rs_files(domain_dir) {
        let content = read_to_string(&file);
        assert!(
            !content.contains("crate::interface") && !content.contains("interface::"),
            "Domain file {:?} must not depend on interface layer",
            file
        );
        assert!(
            !content.contains("crate::infrastructure") && !content.contains("infrastructure::"),
            "Domain file {:?} must not depend on infrastructure layer",
            file
        );
    }
}

#[test]
#[ignore] // Unignore após mover formatação para CLI e remover dependências diretas (ver issues #263, #267)
fn application_must_not_depend_on_interface() {
    let app_dir = Path::new("src/application");
    for file in collect_rs_files(app_dir) {
        let content = read_to_string(&file);
        assert!(
            !content.contains("crate::interface") && !content.contains("interface::"),
            "Application file {:?} must not depend on interface layer",
            file
        );
    }
}

#[test]
#[ignore] // Unignore após remover println! de src/application/**
fn application_must_not_use_println() {
    let app_dir = Path::new("src/application");
    for file in collect_rs_files(app_dir) {
        let content = read_to_string(&file);
        assert!(
            !content.contains("println!"),
            "Application file {:?} must not use println! (use structured returns and let CLI handle logging)",
            file
        );
    }
}

#[test]
fn forbid_anyhow_and_thiserror_in_source_and_cargo_toml() {
    // Project rule: do not use anyhow or thiserror crates in the codebase
    // We ignore Cargo.lock and changelogs to avoid false positives
    let src_dir = Path::new("src");
    for file in collect_rs_files(src_dir) {
        let content = read_to_string(&file);
        assert!(
            !content.contains("anyhow") && !content.contains("thiserror"),
            "File {:?} must not reference anyhow/thiserror",
            file
        );
    }

    // Check Cargo.toml dependencies
    let cargo_toml = Path::new("Cargo.toml");
    let cargo = read_to_string(cargo_toml);
    for banned in ["anyhow", "thiserror"] {
        assert!(
            !cargo.contains(&format!("{} =", banned))
                && !cargo.contains(&format!("{}=", banned))
                && !cargo.contains(&format!("\n{}\n", banned)),
            "Cargo.toml must not include '{}' dependency",
            banned
        );
    }
}
