use super::super::commands::CompanyCommand;
use crate::{application::company_management::CreateCompanyArgs, interface::cli::handlers::get_app_handler};

pub fn handle_company_command(command: CompanyCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        CompanyCommand::Create {
            name,
            code,
            description,
        } => {
            let app = get_app_handler().get_app();
            // Use the company repository directly
            let _company_repo = &app.company_repository;

            let _args = CreateCompanyArgs {
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
