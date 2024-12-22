mod entities;

use std::fs;
use std::path::Path;
use std::process::Command;

use clap::{Arg, Command as ClapCommand, value_parser};
use serde_yml::to_string;

use crate::entities::project::ProjectManifest;


fn main() {
    let matches = ClapCommand::new("ttr")
        .version("0.1.0")
        .author("Seu Nome <seu.email@example.com>")
        .about("Gerenciador de Projetos CLI")
        .subcommand(
            ClapCommand::new("init")
                .about("Inicializa um novo projeto")
                .arg(
                    Arg::new("project_name")
                        .help("Nome do projeto")
                        .required(true)
                        .value_parser(value_parser!(String))
                )
                .arg(
                    Arg::new("template")
                        .short('t')
                        .long("template")
                        .value_name("TEMPLATE")
                        .help("Nome do template (basic, software, etc.)")
                        .value_parser(value_parser!(String)),
                ),
        )
        // Adicione outros subcomandos (new, create, list, etc.)
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("init") {
        // Lógica para o comando `ttr init`

        // 1. Criar a estrutura de diretórios
        create_directories();

        // 2. Criar o arquivo de configuração global (config.yaml)
        create_config_file();

        // 3. Executar o `ttr create project`
        if let Err(e) = create_project(matches) {
            eprintln!("Erro ao criar o projeto: {}", e);
            return;
        }

        // 4. Exibir mensagens de sucesso
        println!("Inicialização concluída com sucesso!");
        println!("Projeto criado com sucesso!");

        // 5. Oferecer dicas e próximos passos
        println!("Próximos passos:");
        println!("  - Adicione recursos com o comando `ttr create resource`");
        println!("  - Crie tarefas com o comando `ttr create task`");
        println!("  - Consulte a documentação para mais informações");
    }
    // Lógica para outros subcomandos (new, create, list, etc.)
}

fn create_directories() {
    // Cria os diretórios ttr-config, projects, etc.
    // Verifica se os diretórios já existem e trata os erros
    if let Err(e) = fs::create_dir_all("ttr-config") {
        eprintln!("Erro ao criar o diretório ttr-config: {}", e);
        return;
    }
    // ... cria outros diretórios
}

fn create_config_file() {
    // Cria o arquivo config.yaml com as configurações padrão
    // Permite que o usuário personalize as configurações
    // Trata os erros de escrita no arquivo
    let config_data = "---
apiVersion: io.tasktaskrevolution/v1alpha1
kind: Config
metadata:
  name: global-config
spec:
  currency: USD
  work_hours_per_day: 8
  work_days_per_week: [monday, tuesday, wednesday, thursday, friday]
  date_format: yyyy-mm-dd
  default_task_duration: 8
  locale: en-US
";
    if let Err(e) = fs::write("ttr-config/config.yaml", config_data) {
        eprintln!("Erro ao criar o arquivo config.yaml: {}", e);
        return;
    }
}

fn create_project(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    // Lógica para criar o projeto
    // Extrai o nome do projeto dos argumentos
    // Chama o comando `ttr create project` (que ainda precisa ser implementado)
    // Trata os erros
    let project_name = matches.get_one::<String>("project_name").map_or("default-project", |v| v.as_str());

    // Executa o comando `ttr create project`
    let output = Command::new("ttr")
        .arg("create")
        .arg("project")
        .arg(project_name)
        .output()?;

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Erro ao executar o comando ttr create project: {}", error_message).into());
    }

    Ok(())
}

// Implemente outras funções (create_resource, create_task, etc.)
//
// fn main() {
//     let project = ProjectManifest::new(
//         "Meu Projeto".to_string(),
//         Some("2024-12-16".to_string()),
//         Some("2025-01-31".to_string()),
//     );
//
//     let yaml_str = to_string(&project).unwrap();
//
//     println!("{}", yaml_str);
// }
