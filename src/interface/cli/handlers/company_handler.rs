use crate::{
    application::company_management::{CreateCompanyArgs, CreateCompanyUseCase},
    infrastructure::persistence::company_repository::FileCompanyRepository,
};
use super::super::commands::CompanyCommand;

pub fn handle_company_command(command: CompanyCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        CompanyCommand::Create {
            name,
            code,
            description,
        } => {
            let company_repository = FileCompanyRepository::new();
            let create_use_case = CreateCompanyUseCase::new(company_repository);

            let args = CreateCompanyArgs {
                name,
                code,
                description,
            };

            match create_use_case.execute(args) {
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
