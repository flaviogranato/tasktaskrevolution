use crate::{
    application::{
        di::DIFactory,
        init::InitManagerData,
    },
    interface::cli::handlers::DI_HANDLER,
};

pub fn handle_init(
    name: String,
    email: String,
    timezone: String,
    work_hours_start: String,
    work_hours_end: String,
    work_days: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let container = DI_HANDLER.get_container()?;
    let init_service: std::sync::Arc<crate::application::di::InitService> = container.resolve()?;

    let work_days_vec: Vec<String> = work_days
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    let init_data = InitManagerData {
        name,
        email,
        timezone,
        work_hours_start,
        work_hours_end,
        work_days: work_days_vec,
    };

    match init_service.init_manager.execute(init_data) {
        Ok(_) => {
            println!("✅ Project management system initialized successfully!");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize system: {}", e);
            Err(e.into())
        }
    }
}
