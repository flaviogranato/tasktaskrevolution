use crate::{application::init::{InitManagerData, InitManagerUseCase}, interface::cli::handlers::get_app_handler};

pub fn handle_init(
    name: String,
    email: String,
    company_name: String,
    timezone: String,
    work_hours_start: String,
    work_hours_end: String,
    work_days: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = get_app_handler().get_app();
    let config_repo = &app.config_repository;

    let _work_days_vec: Vec<String> = work_days.split(',').map(|s| s.trim().to_string()).collect();

    let init_data = InitManagerData {
        name,
        email,
        timezone,
        work_hours_start,
        work_hours_end,
        company_name,
    };

    // Use the actual InitManagerUseCase
    let init_use_case = InitManagerUseCase::new(Box::new(config_repo.clone()));
    
    match init_use_case.execute(init_data) {
        Ok(_config) => {
            println!("✅ Project management system initialized successfully!");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize system: {}", e);
            Err(Box::new(e))
        }
    }
}
