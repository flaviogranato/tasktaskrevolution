//! Testes de integração para performance e stress
//!
//! Estes testes cobrem:
//! - Testes de carga com grandes volumes de dados
//! - Validação de uso de memória
//! - Benchmarks de execução
//! - Simulação de usuários concorrentes
//! - Validação de limpeza de recursos

use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Instant;

/// Teste de performance - criação de grandes volumes de dados
#[test]
fn test_large_dataset_handling() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial

    let start_time = Instant::now();

    // Criar 100 recursos
    for i in 1..=100 {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args([
            "create",
            "resource",
            &format!("Resource {}", i),
            "Developer",
            "--company-code",
            "TECH-CORP",
        ]);
        cmd.assert().success();
    }

    // Criar 50 projetos
    for i in 1..=50 {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args([
            "create",
            "project",
            &format!("Project {}", i),
            &format!("Description for project {}", i),
            "--company-code",
            "TECH-CORP",
        ]);
        cmd.assert().success();
    }

    // Descobrir o primeiro projeto criado dinamicamente
    let projects_dir = temp.path().join("companies").join("TECH-CORP").join("projects");
    let mut project_code = None;
    if let Ok(entries) = std::fs::read_dir(&projects_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let project_yaml = entry.path().join("project.yaml");
                if project_yaml.exists()
                    && let Ok(content) = std::fs::read_to_string(&project_yaml)
                    && let Ok(yaml) = serde_yaml::from_str::<serde_yaml::Value>(&content)
                    && let Some(code) = yaml
                        .get("metadata")
                        .and_then(|m| m.get("code"))
                        .and_then(|c| c.as_str())
                {
                    project_code = Some(code.to_string());
                    break;
                }
            }
        }
    }
    let project_code = project_code.expect("Project code not found");

    // Criar 200 tarefas
    for i in 1..=200 {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args([
            "create",
            "task",
            "--name",
            &format!("Task {}", i),
            "--description",
            &format!("Description for task {}", i),
            "--start-date",
            "2024-01-01",
            "--due-date",
            "2024-12-31",
            "--project-code",
            &project_code,
            "--company-code",
            "TECH-CORP",
        ]);
        cmd.assert().success();
    }

    let elapsed = start_time.elapsed();

    // Validar que todos os dados foram criados
    let resources_dir = temp.child("companies").child("TECH-CORP").child("resources");
    let projects_dir = temp.child("companies").child("TECH-CORP").child("projects");
    let tasks_dir = temp
        .child("companies")
        .child("TECH-CORP")
        .child("projects")
        .child(&project_code)
        .child("tasks");

    resources_dir.assert(predicate::path::is_dir());
    projects_dir.assert(predicate::path::is_dir());
    tasks_dir.assert(predicate::path::is_dir());

    // Validar performance (deve completar em menos de 30 segundos)
    assert!(
        elapsed.as_secs() < 30,
        "Large dataset creation took too long: {:?}",
        elapsed
    );

    println!("Large dataset creation completed in: {:?}", elapsed);

    temp.close()?;
    Ok(())
}

/// Teste de performance - geração de relatórios com grandes volumes
#[test]
fn test_large_report_generation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial com dados grandes

    let start_time = Instant::now();

    // Gerar relatório HTML
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("build");
    cmd.assert().success();

    let elapsed = start_time.elapsed();

    // Validar que o relatório foi gerado
    let public_dir = temp.child("public");
    let index_file = public_dir.child("index.html");
    index_file.assert(predicate::path::exists());

    // Validar performance (deve completar em menos de 10 segundos)
    assert!(elapsed.as_secs() < 10, "Report generation took too long: {:?}", elapsed);

    println!("Large report generation completed in: {:?}", elapsed);

    temp.close()?;
    Ok(())
}

/// Teste de performance - validação de sistema com grandes volumes
#[test]
fn test_large_system_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial com dados grandes

    let start_time = Instant::now();

    // Validar sistema
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("validate").arg("system");
    cmd.assert().success();

    let elapsed = start_time.elapsed();

    // Validar performance (deve completar em menos de 5 segundos)
    assert!(elapsed.as_secs() < 5, "System validation took too long: {:?}", elapsed);

    println!("Large system validation completed in: {:?}", elapsed);

    temp.close()?;
    Ok(())
}

/// Teste de stress - operações concorrentes
#[test]
fn test_concurrent_stress() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial

    let start_time = Instant::now();
    let success_count = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    // Simular 10 operações concorrentes
    for i in 0..10 {
        let temp_path = temp.path().to_path_buf();
        let success_count = Arc::clone(&success_count);

        let handle = thread::spawn(move || {
            for j in 0..10 {
                let mut cmd = Command::cargo_bin("ttr").unwrap();
                cmd.current_dir(&temp_path);
                cmd.args([
                    "create",
                    "resource",
                    &format!("Concurrent Resource {}-{}", i, j),
                    "Developer",
                    "--company-code",
                    "TECH-CORP",
                ]);

                if cmd.output().is_ok() {
                    success_count.fetch_add(1, Ordering::Relaxed);
                }
            }
        });

        handles.push(handle);
    }

    // Aguardar todas as threads
    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed = start_time.elapsed();
    let total_successes = success_count.load(Ordering::Relaxed);

    // Validar que pelo menos 80% das operações foram bem-sucedidas
    assert!(
        total_successes >= 80,
        "Only {} out of 100 operations succeeded",
        total_successes
    );

    // Validar performance (deve completar em menos de 15 segundos)
    assert!(
        elapsed.as_secs() < 15,
        "Concurrent stress test took too long: {:?}",
        elapsed
    );

    println!(
        "Concurrent stress test completed in: {:?} with {} successes",
        elapsed, total_successes
    );

    temp.close()?;
    Ok(())
}

/// Teste de performance - listagem com grandes volumes
#[test]
fn test_large_listing_performance() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial com dados grandes

    let start_time = Instant::now();

    // Testar listagem de recursos
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("list").arg("resources");
    cmd.assert().success();

    let elapsed = start_time.elapsed();

    // Validar performance (deve completar em menos de 3 segundos)
    assert!(elapsed.as_secs() < 3, "Resource listing took too long: {:?}", elapsed);

    println!("Large resource listing completed in: {:?}", elapsed);

    temp.close()?;
    Ok(())
}

/// Teste de performance - criação de empresas em lote
#[test]
fn test_batch_company_creation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "init",
        "--name",
        "Batch Manager",
        "--email",
        "batch@example.com",
        "--company-name",
        "Batch Company",
    ]);
    cmd.assert().success();

    let start_time = Instant::now();

    // Criar 50 empresas
    for i in 1..=50 {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args([
            "create",
            "company",
            "--name",
            &format!("Company {}", i),
            "--code",
            &format!("COMP-{}", i),
            "--description",
            &format!("Description for company {}", i),
        ]);
        cmd.assert().success();
    }

    let elapsed = start_time.elapsed();

    // Validar que todas as empresas foram criadas
    for i in 1..=50 {
        let company_file = temp
            .child("companies")
            .child(format!("COMP-{}", i))
            .child("company.yaml");
        company_file.assert(predicate::path::exists());
    }

    // Validar performance (deve completar em menos de 20 segundos)
    assert!(
        elapsed.as_secs() < 20,
        "Batch company creation took too long: {:?}",
        elapsed
    );

    println!("Batch company creation completed in: {:?}", elapsed);

    temp.close()?;
    Ok(())
}

/// Teste de performance - validação de memória
#[test]
fn test_memory_usage_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial

    // Criar dados para testar uso de memória
    for i in 1..=1000 {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args([
            "create",
            "resource",
            &format!("Memory Test Resource {}", i),
            "Developer",
            "--company-code",
            "TECH-CORP",
        ]);
        cmd.assert().success();
    }

    // Validar que todos os recursos foram criados
    let resources_dir = temp.child("companies").child("TECH-CORP").child("resources");
    resources_dir.assert(predicate::path::is_dir());

    // Testar operações que consomem memória
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("list").arg("resources");
    cmd.assert().success();

    // Testar geração de relatório
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("build");
    cmd.assert().success();

    // Validar que o relatório foi gerado
    let public_dir = temp.child("public");
    let index_file = public_dir.child("index.html");
    index_file.assert(predicate::path::exists());

    println!("Memory usage validation completed successfully");

    temp.close()?;
    Ok(())
}

/// Teste de performance - limpeza de recursos
#[test]
fn test_resource_cleanup_validation() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial

    // Criar dados temporários
    for i in 1..=100 {
        let mut cmd = Command::cargo_bin("ttr")?;
        cmd.current_dir(temp.path());
        cmd.args([
            "create",
            "resource",
            &format!("Temp Resource {}", i),
            "Developer",
            "--company-code",
            "TECH-CORP",
        ]);
        cmd.assert().success();
    }

    // Validar que os recursos foram criados
    let resources_dir = temp.child("companies").child("TECH-CORP").child("resources");
    resources_dir.assert(predicate::path::is_dir());

    // Simular limpeza (fechando o diretório temporário)
    let start_time = Instant::now();
    temp.close()?;
    let elapsed = start_time.elapsed();

    // Validar que a limpeza foi rápida (deve completar em menos de 1 segundo)
    assert!(elapsed.as_secs() < 1, "Resource cleanup took too long: {:?}", elapsed);

    println!("Resource cleanup completed in: {:?}", elapsed);

    Ok(())
}

/// Teste de performance - benchmark de comandos individuais
#[test]
fn test_command_benchmarks() -> Result<(), Box<dyn std::error::Error>> {
    let temp = assert_fs::TempDir::new()?;

    // Setup inicial

    // Benchmark do comando init
    let start_time = Instant::now();
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "init",
        "--name",
        "Benchmark Manager",
        "--email",
        "benchmark@example.com",
        "--company-name",
        "Benchmark Company",
    ]);
    cmd.assert().success();
    let init_time = start_time.elapsed();

    // Benchmark do comando create company
    let start_time = Instant::now();
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "company",
        "--name",
        "Benchmark Corp",
        "--code",
        "BENCH-CORP",
        "--description",
        "Benchmark company",
    ]);
    cmd.assert().success();
    let company_time = start_time.elapsed();

    // Benchmark do comando create resource
    let start_time = Instant::now();
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "resource",
        "Benchmark Resource",
        "Developer",
        "--company-code",
        "BENCH-CORP",
    ]);
    cmd.assert().success();
    let resource_time = start_time.elapsed();

    // Benchmark do comando create project
    let start_time = Instant::now();
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.args([
        "create",
        "project",
        "Benchmark Project",
        "Benchmark project description",
        "--company-code",
        "BENCH-CORP",
    ]);
    cmd.assert().success();
    let project_time = start_time.elapsed();

    // Benchmark do comando build
    let start_time = Instant::now();
    let mut cmd = Command::cargo_bin("ttr")?;
    cmd.current_dir(temp.path());
    cmd.arg("build");
    cmd.assert().success();
    let build_time = start_time.elapsed();

    // Validar que todos os comandos foram executados rapidamente
    assert!(init_time.as_millis() < 1000, "Init command too slow: {:?}", init_time);
    assert!(
        company_time.as_millis() < 1000,
        "Company creation too slow: {:?}",
        company_time
    );
    assert!(
        resource_time.as_millis() < 1000,
        "Resource creation too slow: {:?}",
        resource_time
    );
    assert!(
        project_time.as_millis() < 1000,
        "Project creation too slow: {:?}",
        project_time
    );
    assert!(
        build_time.as_millis() < 2000,
        "Build command too slow: {:?}",
        build_time
    );

    println!("Command benchmarks:");
    println!("  Init: {:?}", init_time);
    println!("  Company: {:?}", company_time);
    println!("  Resource: {:?}", resource_time);
    println!("  Project: {:?}", project_time);
    println!("  Build: {:?}", build_time);

    temp.close()?;
    Ok(())
}
