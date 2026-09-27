shipped_at: 2026-09-27T15:34:55Z
verdict: APPROVED_WITH_WARNINGS
by: alison amorim

## Learnings
- Escreva cada Verify do DoD como prova de comportamento (teste nomeado, config efetiva, harness congelado por hash): Verify por grep levou o loop ao kill em 15 iterações.
- Depois de um squash-merge, rebaseie a próxima branch sobre origin/main (git rebase --onto origin/main <base-antiga>) antes do PR.
- cargo audit reprova por RUSTSEC-2018-0005 via ddc-hi: o CI precisa de audit.toml com cada exceção justificada, não de um audit desligado.
- Teste de hardware fica #[ignore] + DDC_HW_TESTS=1 e só faz escrita Safe reversível, restaurada no Drop.
