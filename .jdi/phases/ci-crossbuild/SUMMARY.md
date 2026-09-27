# SUMMARY — ci-crossbuild (loop iter 1)

8/8 tasks concluídas, 0 bloqueadas. O CI passa nos 4 jobs (Linux e Windows), e os 8 `Verify:` do DoD, rodados literalmente do CONTEXT, imprimem OK no HEAD `ea9136e`.

> Escrito pelo orquestrador a partir do relatório do doer. O harness recusou a escrita do SUMMARY pelo subagente.

## PRs
- github-workflows: [slipalison/github-workflows#13](https://github.com/slipalison/github-workflows/pull/13), branch `qualidade-rust-windows`, head `afc20cc9f5457d045640df903e0ce9a7df1fb515`. O CI de lá passou nos dois heads (runs 36341458451 e 36342435812).
- ddc-control: [slipalison/ddc-control#10](https://github.com/slipalison/ddc-control/pull/10), em rascunho.

## Runs do ddc-control
| Run | Commit | Resultado | O que mostrou |
|---|---|---|---|
| 36341518707 | `6a7b784` (gw `2335b52`) | success | Windows verde de primeira. O runner já tem `python3`, então a ponte saiu do gw (`afc20cc`). Sem cache: Windows 13 min, Linux 7 min. |
| 36342567853 | `f539bba` (gw `afc20cc`) | success | Convergência da T-6; ensaio dos Verify 1–6 OK. |
| 36342845774 | `75c0b8e` (NEG) | failure | O passo `cargo audit` falhou citando RUSTSEC-2018-0005. `Cobertura` também falhou, porque os testes foram pulados e o piso de 80 não achou relatório; é o esperado. |
| 36343108684 | `44fab10` | success | Run de evidência (`RUN_ID`). |

## O que o Windows revelou
- Clippy `-D warnings` limpo.
- 11 binários com `test result: ok.`, o mesmo número do Linux.
- 128 testes do `ddc_tray` no Windows (139 no Linux), incluindo os 3 de clique do `notification_area`.
- Nenhum `STATUS_ENTRYPOINT_NOT_FOUND`: a T-2 (tauri#13419) evitou o problema antes do primeiro build.
- O release do `ddc-tray` compila no Windows.
- O `windows-latest` já tem `python3` no Git Bash (3.12.10 do tool cache), então a premissa A-7 estava desatualizada.
- O harden-runner roda no Windows.
- Cobertura no run de evidência: Linux 82,93% (piso de 80 aprovado), Windows 87,93% (só relata). No gate local, 83,36%.
- Pacotes apt que bastam: `libwebkit2gtk-4.1-dev libudev-dev`. O `libudev-dev` não vem na imagem.

## Desvios decididos
- A ponte `python3` saiu do gw (`fix(qualidade)`): o run mediu que ela não era necessária.
- Cabeçalho do `.cargo/audit.toml` corrigido (`ea250ee`, antes do commit negativo): tirar uma das 3 advisories de AVISO deixa o `cargo audit` sair 0, e só a vulnerabilidade reprova. O procedimento de reavaliação diz isso, e o README acompanha.
- `timeout: 60` mantido, com os tempos medidos no comentário (`3163a00`).
- Aceite da T-2: o literal `grep -c MANIFEST = 0` dá 6 no Linux, porque o `-vv` imprime `CARGO_MANIFEST_*`. A medida usada foi `grep -c -- /MANIFEST` = 0.

## Fora do escopo (candidatos a todo)
- O job `node-ui` sai verde, mas com a anotação `Nenhum relatorio de cobertura encontrado` do `relatar-cobertura`, que já existia no gw.
- O `ubuntu-latest` passa para o Ubuntu 26 a partir de 2026-10-19, e os nomes dos pacotes apt podem mudar.

## Deferred (humano)
1. Merge do github-workflows#13.
2. Trocar `@afc20cc…` por `@main` nos `uses:` do `ci.yml`, e depois ver um run verde.
3. Revisar o diff do `qualidade.yml` e os READMEs.
4. Tirar o #10 do rascunho.

## Testes
- CI Rust: Linux 386 aprovados, Windows 374, com 11 binários em cada.
- UI: `node --test` 161/161; Playwright 138 aprovados e 6 pulados (screenshots).
- Gates locais: fmt, clippy com `--all-features`, `cargo test`, `npm test` e `cargo audit` verdes; cobertura local 83,36%.

## Blocked tasks
- nenhuma
