use serde_yml::to_string;
use std::fs;
use std::path::Path;

use crate::domain::resource::resource::ResourceManifest;

pub fn create_resource(
    name: &String,
    resource_type: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    let resource_name = format!("{}.yaml", name);
    let resources_path = Path::new("resources");

    if !resources_path.exists() {
        match fs::create_dir(resources_path) {
            Ok(_) => println!("Criado o diretório de resources"),
            Err(e) => println!("Erro ao criar diretório de resources: {}", e),
        }
    }

    let resource_path = resources_path.join(resource_name);
    let resource = ResourceManifest::basic(name.to_string(), resource_type.to_string());

    let resource_yaml = to_string(&resource)?;

    fs::write(resource_path, resource_yaml)?;

    println!("Recurso {} criado.", name);
    Ok(())
}
