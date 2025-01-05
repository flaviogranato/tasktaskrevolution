use serde_yml::to_string;
use std::{fs, path::Path};

use crate::domain::project::project::ProjectManifest;

pub fn create_project(
    path: &Path,
    name: &String,
    description: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let project_path = path.join(name);
    let project_file_path = project_path.join("project.yaml");

    let project = ProjectManifest::new(None, name.to_string(), description.clone(), None, None);

    let project_yaml = to_string(&project)?;
    fs::create_dir_all(project_path.clone())?;
    fs::write(project_file_path, project_yaml)?;

    println!("Projeto criado em: {}", project_path.display());
    Ok(())
}
