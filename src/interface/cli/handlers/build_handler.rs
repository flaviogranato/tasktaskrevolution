use crate::application::build_use_case::BuildUseCase;
use std::path::PathBuf;

pub fn handle_build(output: PathBuf, base_url: String) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let build_use_case = BuildUseCase::new(current_dir, output.to_str().unwrap_or("dist"))?;

    match build_use_case.execute() {
        Ok(_) => {
            println!("✅ Static site built successfully!");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Failed to build static site: {}", e);
            Err(e)
        }
    }
}
