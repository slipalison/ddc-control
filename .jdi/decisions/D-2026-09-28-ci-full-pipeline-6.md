D-2026-09-28-ci-full-pipeline-6 (2026-09-28): O `Portao` como check obrigatório é decisão do usuário.

- O ruleset "main so por PR" exige PR com 1 aprovação e nenhum status check: hoje, um PR com o CI vermelho é mergeável.
- A exigência que a esteira pede, ver "O mínimo exigido" no README do gw, é o check `esteira / Portao` no ruleset. É mudança de configuração do repositório: vai ao usuário como recomendação, e é aplicada só com o OK dele.
