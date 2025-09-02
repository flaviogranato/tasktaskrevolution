# Changelog

All notable changes to this project will be documented in this file.

## [0.5.3] - 2025-09-02

### 🚀 Features

- Fase 1 - Estrutura base e nomenclatura idiomática
- Fase 2 - Sistema de erros base estruturado
- Fase 2 - Tipos de erro específicos por domínio
- Fase 2 - Migração da camada de infraestrutura
- Fase 2 - Migração da camada de aplicação
- Fase 2 - Atualizações de módulos e estrutura
- Implement idiomatic Rust patterns and fix all compilation issues
- Implement TDD for domain validations
- Implementa Fase 2 da Opção B - Testes de Integração de Repositórios
- Implementa Fase 3 da Opção B - Testes de Integração para ResourceRepository e TaskRepository
- Implementa Fase 2 da Opção B - Implementação com TDD
- Implementa primeira parte da Opção A - Remove Task Dependency com TDD
- Implementar otimizações zero-copy - Fase 1 completa
- Add comprehensive test coverage for domain shared errors module
- Add comprehensive test coverage for CLI interface module
- Add comprehensive test coverage for vacation validation module
- Add comprehensive test coverage for infrastructure errors module
- Implement domain validations and business rules for company settings
- Implement comprehensive test coverage for domain error modules (FASE 1)
- Complete test coverage for domain::shared::errors (FASE 1 - 4/11)
- Implement comprehensive test coverage for domain::shared::command (FASE 1 - 5/11)
- Implement comprehensive test coverage for domain::shared::convertable (FASE 1 - 6/11)
- Implement comprehensive test coverage for domain::shared::factory (FASE 1 - 7/11)
- Implement comprehensive test coverage for domain::shared::observer (FASE 1 - 8/11)
- Implement comprehensive test coverage for domain::shared::repository
- Implement comprehensive test coverage for domain::shared::specification
- Implement comprehensive test coverage for domain::shared::validatable
- Significantly improve test coverage for domain::shared::command
- Significantly improve test coverage for domain::shared::convertable
- Significantly improve test coverage for domain::shared::factory
- Improve test coverage for application::validate::vacations from 48.94% to 93.62% (+44.68%)
- Improve test coverage for application::task::remove_dependency from 57.78% to 77.78% (+20.00%)
- Improve test coverage for application::create::vacation from 62.50% to 66.67% (+4.17%)
- Achieve 100% test coverage for application::build_use_case (90.36% → 100.00% +9.64%)
- Improve test coverage for application::create::task (80.56% → 94.44% +13.89%)
- Improve test coverage for application::task::assign_resource (75.76% → 78.95% +3.19%)
- Comprehensive test coverage improvements and system enhancements
- Implement company settings management system with CLI and YAML support
- Implement core project entities with TDD approach
- Implement task dependency system with critical path analysis
- Implement comprehensive resource allocation system
- Implement comprehensive reporting and dashboard system
- Complete core project management domain implementation - Add project_new.rs with comprehensive Project entity - Add dependencies.rs with critical path calculation - Add resource_allocation.rs with resource management - Add reporting.rs with dashboard and metrics - Clean up old files and documentation
- Implement comprehensive E2E testing framework for TTR CLI - Add CLI runner utility for executing commands in isolated environments - Add file assertion utilities for validating YAML, CSV, and HTML files - Add project lifecycle test scenarios covering creation, tasks, resources, and export - Add error handling test scenarios for validation and edge cases - Add modular test structure with utilities, scenarios, and integration tests - Add shell script for easy test execution with colored output - Add Cargo.toml configuration for E2E test dependencies - Organize tests in example/e2e_tests/ directory for easy access
- Implement ttr init command for manager/consultant configuration - Add InitManagerUseCase with comprehensive validation - Add CLI support for ttr init with required and optional parameters - Support manager name, email, company name, timezone, work hours - Add validation for empty fields and invalid email format - Update CLI tests to match new command structure - Fix project compilation issues to enable init command testing
- Implement CRUD foundation for Company entity
- Implement YAML persistence for Company entity with Kubernetes/Backstage pattern
- Implement automatic code generation for Company entities
- Implement complete CRUD operations for Company entities
- Improve company describe formatting to match kubectl style
- Convert company CLI commands to English and remove comments
- Convert Project/Resource/Task CLI commands to English
- Verify CLI formatting consistency (no changes needed)
- Implement comprehensive validation system architecture
- Implement DDD specification pattern in validation system
- Implement comprehensive typestate pattern for project management
- Implement resource assignment to tasks - add with_assigned_resources method to AnyTask - implement assign_resource_to_task in AnyProject - fix assign resource to task use case
- Implement project update functionality - add set_name and set_description methods to AnyProject - implement UpdateProjectUseCase logic - enable test_update_project_name_and_description_success
- Implement task cancellation functionality - add cancel method to AnyTask - add cancel_task method to AnyProject - fix AnyProject::add_task to use task code instead of placeholder - implement DeleteTaskUseCase to properly cancel tasks - fix test_cancel_task_success
- Implement task dependency management and update operations

### 🐛 Bug Fixes

- Correct unit tests and use case implementation
- Aplicar correções do clippy e resolver warnings críticos
- Resolve compilation errors in E2E test framework - Fix string repeat syntax errors in mod.rs - Fix import paths between modules - Fix ownership issues in project_lifecycle.rs - Remove duplicate test targets to avoid conflicts - Clean up unused imports and warnings - All tests now compile successfully (9 passed, 2 failed due to missing TTR CLI)
- Clean up E2E test framework and resolve compilation issues - Remove duplicate test targets to avoid conflicts - Fix ownership issues in project_lifecycle.rs - Clean up unused imports and warnings - Update .gitignore to allow e2e_tests directory - All tests now compile successfully (9 passed, 2 failed due to missing TTR CLI) - Framework ready for CLI implementation
- Resolve mutable borrowing issue in assign_resource_to_task - separate read and write operations to avoid conflicts - fix compilation error E0499
- Resolve build_use_case template rendering issues
- Resolve all remaining test failures
- Resolve all clippy warnings and compilation errors

### 🚜 Refactor

- Clean up unused imports and reduce warnings
- Simplify builder pattern while maintaining typestate
- [**breaking**] Unify Project entities and simplify project management
- Remove unnecessary dependencies and simplify architecture
- Migrate from thiserror to idiomatic Rust error handling

### 📚 Documentation

- Add batch operations planning for future implementation
- Comprehensive roadmap with tool analysis and feature recommendations
- Move roadmap to docs directory and translate to Portuguese

### 🧪 Testing

- Fix first test in task report - remove ProjectBuilder argument and handle Result properly
- Fix all ProjectBuilder tests in task report - complete file cleanup
- Fix all ProjectBuilder tests in project repository - complete file cleanup
- Fix all ProjectBuilder tests in list projects - complete file cleanup
- Fix all ProjectBuilder tests in list tasks - complete file cleanup
- Fix all ProjectBuilder tests in project assign resource to task - complete file cleanup
- Fix all ProjectBuilder tests in create project - complete file cleanup
- Fix ProjectBuilder tests in create task - complete file cleanup
- Fix all ProjectBuilder tests in cancel project - complete file cleanup
- Fix all ProjectBuilder tests in describe project - complete file cleanup
- Fix all ProjectBuilder tests in vacation report - complete file cleanup
- Fix pattern matching in task assign resource - complete file cleanup
- Fix all ProjectBuilder tests in task describe task - complete file cleanup
- Fix all remaining compilation errors in test suite - complete project cleanup
- Fix create task tests - use iterator instead of hardcoded task codes
- Fix list tasks test - use ProjectBuilder correctly to add tasks

### ⚙️ Miscellaneous Tasks

- Exclude docs folder from git tracking
- Prepare code for v0.5.3 release
- Bump version to 0.5.3

## [0.5.1] - 2025-08-03

### 🐛 Bug Fixes

- *(build)* Restore project discovery logic to correctly find and build projects

### ⚙️ Miscellaneous Tasks

- *(release)* Prepare for version 0.5.1

## [0.5.0] - 2025-08-01

### 🚀 Features

- *(ui)* Refactor HTML templates with Tailwind CSS
- *(cli)* Add 'task assign' command
- *(domain)* Standardize on UUIDv7 for all entity IDs
- Implement universal UUIDs and sequential codes
- *(cli)* Add list commands for projects, resources, and tasks
- Implement full CRUD for projects, resources, and tasks
- Add `describe` command for project, resource, task, and config

### 🐛 Bug Fixes

- *(build)* Handle multiple projects correctly in BuildUseCase
- Correctly load tasks when describing a project

### 🚜 Refactor

- *(task)* Implement typestate pattern for Task
- *(resource)* Implement typestate pattern for Resource
- *(resource)* Implement typestate pattern and fix subsequent issues
- *(project)* Implement typestate pattern for Project
- *(domain)* Move Any* enums to dedicated modules
- *(performance)* Reduce clones by using string slices
- *(application)* Organize use cases into functional submodules
- Integrate task management into project aggregate

### 📚 Documentation

- *(readme)* Update to reflect current project functionality
- *(contributing)* Simplify and translate contribution guidelines

### 🎨 Styling

- Apply automatic formatting
- Apply clippy fixes and remove warnings

### 🧪 Testing

- *(manifests)* Restore and expand test coverage

### ⚙️ Miscellaneous Tasks

- *(changelog)* Generate changelog with git-cliff
- *(release)* Prepare for version 0.5.0

## [0.4] - 2025-07-29

### 🚀 Features

- *(cargo)* Added release profile
- Adiciona validação de datas para férias
- Implementa sistema de tarefas e correção de erros de compilação
- *(refactor)* [**breaking**] Reorganize project structure for better modularity and DDD alignment
- *(task)* Added task management
- Refatora templates de tarefas e atualiza manifesto
- *(project)* Implementa herança de timezone da configuração global
- Busca dinâmica de projeto e exibição de férias
- Refatora UI para HTML puro e implementa páginas de detalhes
- Adicionar arquivos de template do site

### 🐛 Bug Fixes

- Corrige importações e tipos de parâmetros para testes
- *(ci)* Correct YAML validation for GitLab CI using Nushell
- Fixed some tests
- Fixed tests and cargo clippy
- Removed unnecessary code
- Cargo-audit
- Binary name
- *(task-builder)* Propagate assigned_resources, fix Option<NaiveDate> handling, and update tests
- *(build)* Correct HTML generation and navigation links
- Corrige inconsistências e redundâncias nos manifestos
- *(task)* Garante a persistência correta do project_code
- *(manifests)* Evita valores nulos em campos opcionais
- *(templates)* Remove '$' extra em interpolações Vue.js

### 🚜 Refactor

- Ajustes nas implementações dos casos de uso e repositórios
- Corrige todos os avisos do Clippy e configura lints para ignorar avisos específicos
- Remove TaskService da camada de aplicação
- *(ci)* Remove sccache and optimize GitLab CI pipeline for free runners
- Migrated the structure do respect Rust Edition 2024
- *(core)* Overhaul test suite and apply code quality improvements
- *(manifests)* Padroniza apiVersion com constante
- Corrigir avisos do clippy e refatorar a criação de projetos e tasks

### 📚 Documentation

- Adiciona licença Creative Commons BY-SA 4.0
- Melhora a documentação do README com instruções de uso e licença CC BY-NC-SA 4.0
- Adiciona guia de contribuição com padrões e diretrizes do projeto
- Atualização do changelog
- Atualiza changelog com remoção de TaskService
- Atualiza changelog com correção dos testes

### 🎨 Styling

- *(all)* Format all files
- Reformating code
- Reformating
- *(all)* Formatting code

### ⚙️ Miscellaneous Tasks

- *(dependencies)* Bump dependencies
- *(github)* Remove unnecessary file
- *(cargo)* Update cargo.lock
- *(dependencies)* Updated dependencies
- Adiciona pipeline GitLab CI para build, teste e release
- Adiciona anotações globais para suprimir avisos de código não utilizado
- *(github)* Removed unnecessary github configuration
- Pipeline
- Pipeline
- Removed unnecessary files
- Integrate git-cliff for automatic CHANGELOG.md generation
- Removed unnecessary configuration
- Bump version
- Removed unnecessary text
- Updated dependencies
- Ignora o diretório 'example' no .gitignore
- Adiciona workflow do GitHub Actions para Integração Contínua
- Configurar workflow para criar release em tags
- Adicionar permissão para criar releases

### 🛡️ Security

- *(ci)* Enhance pipeline with parallel builds, changelog automation, and advanced compression

### Build

- *(ci)* Improve pipeline with advanced compression and changelog automation

## [0.3.0] - 2025-02-26

### 🚀 Features

- *(task)* Implement task domain entity and repository
- *(task)* Implement task creation and persistence
- *(manifests)* Standardize manifest formats and add timezone support
- *(report)* Added vacation's report
- *(task)* Added task
- *(pipeline)* Added action to create a release

### 🚜 Refactor

- *(task)* Move serialization and repository to infrastructure layer

### ⚙️ Miscellaneous Tasks

- *(cargo)* Updated cargo.lock

## [0.2.0] - 2025-02-17

### 🚀 Features

- *(cli)* Moving foward to adopt DDD
- *(main)* Moved to use DDD as in the books
- *(validate)* Added vacation overlap validation
- Adiciona comando para criar período de férias
- Adiciona histórico de horas extras

### 🐛 Bug Fixes

- *(config)* Fixed create initial config file
- *(resource)* Fixed create resource dir

### 🚜 Refactor

- *(all)* Moved to the respective folders
- *(application)* Moved the creation from cli to application
- *(config)* Refactoring config to better use DDD techniques
- Migra de serde_yml para serde_yaml e corrige testes

### ⚙️ Miscellaneous Tasks

- *(doc)* Added the new files and folders organization
- *(gitignore)* Removed test folder
- *(config)* Removed code commented from the file.
- *(resource)* Catch the return of write the file

## [0.1.0] - 2025-01-01

### 🚀 Features

- *(project)* Improved serialization
- *(resource)* Added resource creation
- *(config)* Made config::spec dont required
- *(validation)* Initial struct of validation command

### 🚜 Refactor

- *(main)* Moving to DDD

### ⚙️ Miscellaneous Tasks

- *(project)* Normalized the fields of project
- *(tests)* Added speculate lib to BDD tests
- *(cliff)* Bump version

## [0.0.1] - 2024-12-24

### 🚀 Features

- *(project)* Added creation projects
- *(main)* Added changelog file and configuration git-cliff

### 🚜 Refactor

- *(main)* Refactoring all project and create config entity

### ⚙️ Miscellaneous Tasks

- *(main)* Initial commit

<!-- generated by git-cliff -->
