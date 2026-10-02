D-2026-10-01-input-switch-autostart-3 (2026-10-01): Quinta rodada do critic (iteração 5, fim da rodada 1 do loop; auto-reset 1/3). Linhas 1 e 8 ocas com prova objetiva; linhas 3 e 4 com lacuna não objetiva (W-19), fechada junto porque inverte o sentido do toast para o usuário.

(a) Linha 1, "as constantes são PRIVADAS de `retry.rs`, só alimentam o `Default`, `worker.rs` não as nomeia, só a política manda". O Verify não conferia isso: tornar `INPUT_SETTLE_WINDOW` `pub(super)` e travar a janela nela em `worker.rs` passava, porque nenhum teste usava janela maior que 1 s. Duas provas novas:
- o Verify exige que o CONJUNTO das linhas que não são comentário e citam `INPUT_SETTLE_STEP`/`INPUT_SETTLE_WINDOW` em `crates` e `apps` seja exatamente as 2 declarações `const` (sem `pub`) e os 2 usos no `Default`, todos em `retry.rs`;
- a política CUSTOM de (b) e (e) passa a ter janela MAIOR que a do `Default` (por exemplo, passo 500 ms e janela 4 s, no relógio virtual, sem custo de tempo real), então um worker que travasse a janela na do `Default` falha em (b).

(b) Linha 8, "NÃO têm código de teste embutido". Um exemplo de rustdoc (```` ``` ```` em `///`) roda como teste no `cargo test --workspace` e escapava da varredura. O `ddc-adapters` tem 0 doctests hoje, então ganha `[lib] doctest = false` e nada se perde. O Verify confere a configuração EFETIVA (`cargo metadata`: `doctest == false` no alvo lib). A regex dos 3 arquivos passa a pegar QUALQUER linha com `cfg` (inclusive `#[cfg(all(` multilinha gerado pelo rustfmt e `cfg(doctest)`) e qualquer atributo que contenha `test` (inclusive `#[ test ]`). Cada ocorrência tem de ser `#[cfg(test)]` seguida de `mod tests;`, uma por arquivo, o que fecha a W-18.

(c) Linhas 3 e 4 (W-19). Os nomes apareciam, mas nenhum teste fixava QUAL é o mantido e qual é o pedido: um template com os papéis trocados passava. Os testes passam a conferir o texto INTEIRO por igualdade com frases LITERAIS, independentes do template, em `en` e `pt-BR`, para o aviso de input e para o genérico:
- no view-model, os 4 testes nomeados;
- no spec, os toasts `(en)`, `(pt-BR)`, o genérico e o de leitura que falha.

`tests/ui/view-model.test.mjs` passa a ser congelado por SHA-256 na linha 3. O spec é recongelado na linha 4, e os arquivos de teste da linha 1 também.
