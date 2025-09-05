use crate::{
    application::{
        company_management::CreateCompanyArgs,
        di::CreateUseCaseService,
    },
    interface::cli::handlers::DI_HANDLER,
};
use super::super::commands::CompanyCommand;

pub fn handle_company_command(command: CompanyCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        CompanyCommand::Create {
            name,
            code,
            description,
        } => {
            let container = DI_HANDLER.get_container()?;
            let create_service: std::sync::Arc<CreateUseCaseService> = container.resolve()?;

            let args = CreateCompanyArgs {
                name,
                code,
                description,
            };

            match create_service.create_company.execute(args) {
                Ok(company) => {
                    println!("✅ Company created successfully!");
                    println!("   Name: {}", company.name());
                    println!("   Code: {}", company.code());
                    if let Some(desc) = company.description() {
                        println!("   Description: {}", desc);
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to create company: {}", e);
                    Err(e.into())
                }
            }
        }
    }
}
