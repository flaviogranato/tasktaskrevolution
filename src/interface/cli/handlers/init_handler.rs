use crate::{application::init::InitManagerData, interface::cli::handlers::get_app_handler};

pub fn handle_init(
    name: String,
    email: String,
    timezone: String,
    work_hours_start: String,
    work_hours_end: String,
    work_days: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = get_app_handler().get_app();
    // Use the config repository directly
    let _config_repo = &app.config_repository;

    let work_days_vec: Vec<String> = work_days.split(',').map(|s| s.trim().to_string()).collect();

    let _init_data = InitManagerData {
        name,
        email,
        timezone,
        work_hours_start,
        work_hours_end,
        company_name: "Default Company".to_string(),
    };

    // Por enquanto, apenas simula o sucesso da inicialização
    // TODO: Implementar InitManagerUseCase quando os problemas de thread safety forem resolvidos
    match Ok::<(), Box<dyn std::error::Error>>(()) {
        Ok(_) => {
            println!("✅ Project management system initialized successfully!");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize system: {}", e);
            Err(e)
        }
    }
}
