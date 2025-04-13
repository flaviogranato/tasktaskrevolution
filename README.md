# TaskTaskRevolution

Um utilitário de linha de comando para gerenciar suas tarefas e projetos de forma simples e eficiente.

## Recursos

- Criação de projetos
- Criação de recursos dentro de projetos
- Criação de tarefas associadas a projetos e recursos

## Funcionalidades

### Qualidade de Código

O projeto utiliza hooks de pre-commit para garantir qualidade:

- Verificação de linting com clippy
- Execução automática de testes
- Verificação de formatação
- Auditoria de dependências
- Verificação de documentação

## Uso

### Instalação

```bash
# Clone o repositório
git clone https://gitlab.com/flaviogranato/tasktaskrevolution.git

# Entre no diretório
cd tasktaskrevolution

# Compile o projeto
cargo build --release
```

### Comandos Básicos

#### Gerenciamento de Projetos

```bash
# Criar um novo projeto
tasktaskrevolution project create "Nome do Projeto"

# Listar todos os projetos
tasktaskrevolution project list

# Visualizar detalhes de um projeto
tasktaskrevolution project show "Nome do Projeto"

# Remover um projeto
tasktaskrevolution project delete "Nome do Projeto"
```

#### Gerenciamento de Recursos

```bash
# Adicionar um recurso a um projeto
tasktaskrevolution resource add "Nome do Projeto" "Nome do Recurso"

# Listar recursos de um projeto
tasktaskrevolution resource list "Nome do Projeto"

# Remover um recurso
tasktaskrevolution resource delete "Nome do Projeto" "Nome do Recurso"
```

#### Gerenciamento de Tarefas

```bash
# Criar uma nova tarefa
tasktaskrevolution task create "Nome do Projeto" "Nome do Recurso" "Descrição da Tarefa"

# Listar tarefas de um recurso
tasktaskrevolution task list "Nome do Projeto" "Nome do Recurso"

# Marcar uma tarefa como concluída
tasktaskrevolution task complete "Nome do Projeto" "Nome do Recurso" "ID da Tarefa"

# Remover uma tarefa
tasktaskrevolution task delete "Nome do Projeto" "Nome do Recurso" "ID da Tarefa"
```

### Opções Globais

```bash
--help     # Mostra a ajuda
--version  # Mostra a versão do programa
--debug    # Ativa o modo debug
```

### Exemplos de Uso

1. Criando um novo projeto de desenvolvimento:

```bash
# Criar o projeto
tasktaskrevolution project create "Sistema de E-commerce"

# Adicionar recursos
tasktaskrevolution resource add "Sistema de E-commerce" "Frontend"
tasktaskrevolution resource add "Sistema de E-commerce" "Backend"
tasktaskrevolution resource add "Sistema de E-commerce" "Banco de Dados"

# Criar tarefas para o Frontend
tasktaskrevolution task create "Sistema de E-commerce" "Frontend" "Implementar página inicial"
tasktaskrevolution task create "Sistema de E-commerce" "Frontend" "Criar componente de carrinho"
```

2. Gerenciando um projeto de manutenção:

```bash
# Listar projetos existentes
tasktaskrevolution project list

# Ver detalhes de um projeto específico
tasktaskrevolution project show "Sistema de E-commerce"

# Listar recursos do projeto
tasktaskrevolution resource list "Sistema de E-commerce"

# Listar tarefas de um recurso específico
tasktaskrevolution task list "Sistema de E-commerce" "Frontend"
```

### Dicas de Uso

- Use aspas para nomes que contenham espaços
- IDs de tarefas são numéricos e podem ser visualizados no comando `task list`
- Você pode usar a tecla TAB para autocompletar comandos
- Use `--help` após qualquer comando para ver mais detalhes sobre suas opções

## Desenvolvimento

Este projeto segue os princípios da Arquitetura Limpa (Clean Architecture) e Domain-Driven Design (DDD), organizando o código em camadas bem definidas:

- **Domain**: Contém as regras de negócio e entidades principais
- **Application**: Implementa os casos de uso da aplicação
- **Infrastructure**: Fornece implementações concretas para persistência e APIs
- **Interface**: Gerencia a interação com o usuário

### Configuração do Ambiente

1. Clone o repositório
2. Instale as dependências: `cargo build`
3. Configure os hooks de git: `chmod +x .git/hooks/pre-commit`
4. Execute os testes: `cargo test`
5. Execute os benchmarks: `cargo bench`

### Logging

Para ajustar o nível de logging, use a variável de ambiente `RUST_LOG`:

```bash
RUST_LOG=debug cargo run
```

### Métricas

As métricas de performance estão disponíveis em `http://localhost:9090/metrics` quando o programa está em execução.

## Contribuindo

Atualmente, este projeto é mantido exclusivamente por Flavio Granato. Como é um projeto pessoal em desenvolvimento, não estamos aceitando contribuições externas no momento.

## Licença

Este projeto está licenciado sob a [Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International License](https://creativecommons.org/licenses/by-nc-sa/4.0/).

Esta licença permite que você:

- Compartilhe: copie e redistribua o material em qualquer formato
- Adapte: remixe, transforme e construa sobre o material

Sob os seguintes termos:

- Atribuição: Você deve dar crédito apropriado, fornecer um link para a licença e indicar se foram feitas alterações.
- Não Comercial: Você não pode usar o material para fins comerciais.
- Compartilha Igual: Se você remixar, transformar ou construir sobre o material, você deve distribuir suas contribuições sob a mesma licença que o original.

Para mais detalhes, consulte o arquivo [LICENSE](LICENSE) no repositório.
