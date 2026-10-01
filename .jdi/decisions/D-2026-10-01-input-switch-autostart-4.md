D-2026-10-01-input-switch-autostart-4 (2026-10-01): Sexta rodada do critic (rodada 2, iteração 1). As linhas 1 e 10 estão ocas com prova objetiva, e a linha 8 com uma lacuna não objetiva, fechada junto porque é barata.

(a) Linha 1 (W-20, regressão da D-2026-10-01-input-switch-autostart-3). A janela custom de (b)/(e) passou de 1 s para 4 s, e com isso nenhum teste determinístico ficou com janela abaixo de um piso. Um piso "defensivo" em `settle_input` passa a frio:
- `settle.window.max(...)` com o `Default`;
- ou com uma constante de outro nome (`MIN_SETTLE_WINDOW`).

O Verify 1 dá `OK` em 16 de 20 rodadas, porque só uma corrida no (f) o pega. Para fechar, a 1ª parte de (f) ganha um limite de cima DETERMINÍSTICO no número de leituras: `ceil(janela / passo)`. Como `sleep` nunca volta antes do pedido, qualquer piso acima da janela injetada (100 ms) excede o limite. `worker/tests.rs` é recongelado.

(b) Linha 10. A phase passou a tocar `crates/ddc-adapters/Cargo.toml`, que ficava fora do pathspec. A lista de arquivos varridos passa a ser a UNIÃO de duas fontes:
- os caminhos fixos de hoje, mais `crates/ddc-adapters/Cargo.toml`;
- os arquivos não-Rust que o diff da phase toca (`git diff --name-only` contra a base, sem `.jdi`, `Cargo.lock`, `fixtures` e os 2 arquivos de tradução).

Assim, qualquer arquivo não-Rust novo da phase entra sozinho na varredura.

(c) Linha 8 (W-21, não objetiva). Um `#[test]` gerado por macro, ou um macro de tabela definido fora dos 3 arquivos e chamado sem `cfg` dentro deles, escapava da regex. Prova comportamental ANCORADA pela `cargo test -p ddc-adapters --lib -- --list`: todo teste listado sob `ddc_hi_backend::` tem de casar `^ddc_hi_backend::((worker|retry|hardware|identity)::)?tests::<nome>$`. Um teste fora desses módulos de teste, inclusive um `mod x { mod tests { ... } }` inline, falha. A lista não pode vir vazia.
