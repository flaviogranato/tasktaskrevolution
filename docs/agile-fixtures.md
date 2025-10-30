## Agile QA Fixtures

Arquivos de apoio para testes de QA do módulo ágil.

Local: `tests/fixtures/agile/`

- sprint_example.json: Sprint padrão (14 dias, 40 SP).
- burndown_input.json: Conclusões diárias para gerar burndown.
- kanban_board.json: Board com colunas, WIP e algumas tarefas.
- velocity_history.json: Histórico de velocidade para análises.

Como usar (sugestões):
- Burndown: carregar `sprint_example.json`, aplicar `burndown_input.json` no `BurndownCalculator`.
- Kanban: carregar `kanban_board.json` no `KanbanBoardManager` e rodar métricas de fluxo.
- Velocity: iterar `velocity_history.json` e alimentar `VelocityTracker`.


