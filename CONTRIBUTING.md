# Guia de Contribuição

## Sobre o Projeto

O TaskTaskRevolution é um utilitário de linha de comando para gerenciamento de tarefas e projetos, desenvolvido em Rust. O projeto segue os princípios da Arquitetura Limpa (Clean Architecture) e Domain-Driven Design (DDD).

## Estado Atual do Projeto

Atualmente, este projeto está em fase inicial de desenvolvimento e é mantido exclusivamente por Flavio Granato. Como é um projeto pessoal em desenvolvimento, não estamos aceitando contribuições externas no momento.

## Estrutura do Projeto

O projeto está organizado nas seguintes camadas:

- **Domain**: Contém as regras de negócio e entidades principais
- **Application**: Implementa os casos de uso da aplicação
- **Infrastructure**: Fornece implementações concretas para persistência e APIs
- **Interface**: Gerencia a interação com o usuário

## Padrões de Código

Quando o projeto estiver aberto para contribuições, os seguintes padrões devem ser seguidos:

### Estilo de Código

- Siga as convenções de estilo do Rust
- Use `rustfmt` para formatação automática
- Execute `clippy` para verificar boas práticas
- Mantenha as funções pequenas e focadas
- Documente funções públicas com `///`

### Commits

- Use [Conventional Commits](https://www.conventionalcommits.org/)
- Prefixos comuns:
  - `feat:` para novas funcionalidades
  - `fix:` para correções de bugs
  - `docs:` para alterações na documentação
  - `style:` para alterações de estilo
  - `refactor:` para refatorações
  - `test:` para adição ou modificação de testes
  - `chore:` para tarefas de manutenção

### Branches

- `main`: branch principal
- `develop`: branch de desenvolvimento
- `feature/*`: para novas funcionalidades
- `bugfix/*`: para correções de bugs
- `release/*`: para preparação de releases

### Pull Requests

Quando o projeto estiver aberto para contribuições, os PRs devem:

1. Ter uma descrição clara das mudanças
2. Incluir testes para novas funcionalidades
3. Passar em todos os testes
4. Não ter conflitos com a branch base
5. Seguir as convenções de código

## Licença

Este projeto está licenciado sob a [Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International License](https://creativecommons.org/licenses/by-nc-sa/4.0/).

## Contato

Para questões sobre o projeto, entre em contato com Flavio Granato através do GitLab. 