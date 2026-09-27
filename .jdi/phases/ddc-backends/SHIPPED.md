shipped_at: 2026-09-27T15:34:55Z
verdict: APPROVED_WITH_WARNINGS
by: alison amorim

## Learnings
- O ddc-hi 0.4.1 traz serde_yaml 0.7.5 e yaml-rust (RUSTSEC-2018-0005, RUSTSEC-2024-0320) e o nom 3 com aviso future-incompat: fixe a toolchain e trate o cargo audit no CI com exceção justificada.
- Handles de monitor envelhecem: a UI e as hotkeys precisam re-enumerar ao abrir o popup, numa mudança de display ou depois de um Transport.
- Pior caso de uma chamada com id desconhecido é ~7s (fila + enumerate + VCP): nunca chame o backend na thread de UI.
