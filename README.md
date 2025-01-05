# tasktaskrevolution

Um utilitário de linha de comando para gerenciar suas tarefas e projetos de forma simples e eficiente.

## Recursos

- Criação de projetos
- Criação de recursos dentro de projetos
- Criação de tarefas associadas a projetos e recursos

## Uso

... (instruções de uso)

src/
├── domain/
│ ├── shared_kernel/
│ │ ├── metadata.rs # ObjectMeta
│ │ ├── kubernetes_resource.rs # Trait KubernetesResource
│ │ ├── value_objects.rs # Value Objects compartilhados (ex: Name, Quantity)
│ │ ├── errors.rs # Erros genéricos do domínio
│ │ mod.rs
│ ├── order/
│ │ ├── order.rs # Entidade Order
│ │ ├── order_item.rs # Value Object OrderItem
│ │ ├── errors.rs # Erros específicos de Order
│ │ mod.rs
│ ├── product/
│ │ ├── product.rs # Entidade Product
│ │ ├── errors.rs # Erros específicos de Product
│ │ mod.rs
│ ├── deployment/
│ │ ├── deployment.rs # Entidade Deployment
│ │ ├── errors.rs # Erros específicos de Deployment
│ │ mod.rs
│ ├── service/
│ │ ├── service.rs # Entidade Service
│ │ ├── errors.rs # Erros específicos de Service
│ │ mod.rs
│ mod.rs
├── application/
│ ├── create_order.rs # Caso de uso: Criar Pedido
│ ├── list_products.rs # Caso de uso: Listar Produtos
│ mod.rs
├── infrastructure/
│ ├── persistence/
│ │ ├── order_repository.rs # Interface do Repositório de Pedidos
│ │ ├── order_repository_impl.rs # Implementação do Repositório de Pedidos
│ │ ├── product_repository.rs
│ │ ├── product_repository_impl.rs
│ │ mod.rs
│ ├── api/ # Adaptadores de API (se aplicável)
│ │ ├── rest.rs
│ │ mod.rs
│ mod.rs
├── interface/
│ ├── cli.rs # Interface de linha de comando
│ mod.rs
├── config/ # Arquivos de configuração
├── Cargo.toml
├── Cargo.lock
└── main.rs
