# Changelog

All notable changes to this project will be documented in this file.

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
