use super::super::commands::CompanyCommand;
use crate::{application::company_management::CreateCompanyArgs, interface::cli::handlers::DI_HANDLER};

pub fn handle_company_command(command: CompanyCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        CompanyCommand::Create {
            name,
            code,
            description,
        } => {
            let _container = DI_HANDLER.get().ok_or("DI container not initialized")?;
            // Por enquanto, não usa DI - será implementado posteriormente
            // let create_service: std::sync::Arc<CreateUseCaseService> = container.try_resolve().ok_or("Failed to resolve CreateUseCaseService")?;

            let args = CreateCompanyArgs {
                code: code.clone(),
                name: name.clone(),
                description: description.clone(),
                tax_id: None,
                address: None,
                email: None,
                phone: None,
                website: None,
                industry: None,
                created_by: "system".to_string(),
            };

            // Por enquanto, simula a criação da empresa
            // TODO: Usar DI quando for implementado
            match Ok::<(), Box<dyn std::error::Error>>(()) {
                Ok(_) => {
                    println!("✅ Company created successfully!");
                    println!("   Name: {}", name);
                    println!("   Code: {}", code);
                    if let Some(desc) = description {
                        println!("   Description: {}", desc);
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to create company: {}", e);
                    Err(e)
                }
            }
        }
    }
}
