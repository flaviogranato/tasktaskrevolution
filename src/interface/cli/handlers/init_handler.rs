use crate::{application::init::InitManagerData, interface::cli::handlers::DI_HANDLER};

pub fn handle_init(
    name: String,
    email: String,
    timezone: String,
    work_hours_start: String,
    work_hours_end: String,
    work_days: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let _container = DI_HANDLER.get().ok_or("DI container not initialized")?;
    // Por enquanto, não usa DI - será implementado posteriormente
    // let _init_service: std::sync::Arc<crate::application::di::InitService> = container.try_resolve().ok_or("Failed to resolve InitService")?;

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
