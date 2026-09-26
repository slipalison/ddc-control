D-2026-09-26-full-osd-control-1 (2026-09-26): Catálogo MCCS 2.2 em `ddc-core` (novo módulo `domain::mccs_catalog`, sem nova dependência — D-2) consolida NOME, `FeatureKind` (C/NC/T), `Access` (RO/WO/RW), `Risk` (Safe/Dangerous) e nomes de valor NC numa ÚNICA tabela estática `CatalogEntry { code: VcpCode, name: Option<&'static str>, kind: FeatureKind, access: Access, risk: Risk, values: &'static [(u8, &'static str)] }`, exposta por `catalog_entry(code) -> Option<&'static CatalogEntry>`, `catalog_codes() -> &'static [VcpCode]` e `value_name(code, byte) -> Option<&'static str>`.

**Escopo do catálogo:** só os códigos REALMENTE observados no monitor de dev "RTK QHD HDR" (declarados no caps fixture `crates/ddc-core/tests/fixtures/rtk_qhd_hdr_caps.txt` + sondados fora do caps pelo orquestrador em 2026-09-25/26) — não a tabela VESA MCCS 2.2 inteira (~200 códigos teóricos, a maioria nunca vista em hardware real). Rationale: nomear um código nunca confirmado em hardware arriscaria uma string errada indo pro usuário final (o objetivo do card é literalmente controlar ESTE monitor pelo nome); a tabela abaixo é fechada e auditável. Um código fora dela cai no comportamento genérico já existente (`Capabilities::feature`: kind heurístico pelo caps, `access = ReadWrite`, `risk` = Dangerous por fail-safe) — extensão futura do catálogo para outros monitores é todo aberto, não bloqueia esta phase.

**Tabela completa (code | name | kind | access | risk):**
- 0x02 New Control Value | NC | RW | Safe
- 0x04 Restore Factory Defaults | NC | WO | Dangerous (já existente) — values: 00 Reset
- 0x05 Restore Factory Luminance/Contrast Defaults | NC | WO | Dangerous (já existente) — values: 00 Reset
- 0x06 Restore Factory Geometry Defaults | NC | WO | Dangerous (já existente) — values: 00 Reset
- 0x08 Restore Factory Color Defaults | NC | WO | Dangerous (já existente) — values: 00 Reset
- 0x0B Color Temperature Increment | C | RO | Safe
- 0x0C Color Temperature Request | C | RW | Safe
- 0x10 Luminance (Brightness) | C | RW | Safe (já existente)
- 0x12 Contrast | C | RW | Safe (já existente)
- 0x14 Select Color Preset | NC | RW | Safe (já existente) — values: 01 sRGB, 02 Display Native, 04 5000 K, 05 6500 K, 06 7500 K, 08 9300 K, 0B User 1
- 0x16/0x18/0x1A Video Gain Red/Green/Blue | C | RW | Safe (já existentes)
- 0x1E Auto Setup | NC | RW | **Dangerous (novo)** — dispara ajuste automático de geometria sem confirmação prévia do usuário
- 0x20 Horizontal Position | C | RW | **Dangerous (novo)**
- 0x30 Vertical Position | C | RW | **Dangerous (novo)** — H/V Position podem empurrar a imagem pra fora da área visível sem uma forma óbvia de reverter às cegas (mesma classe de risco que geometria/reset)
- 0x52 (sem nome confiável — semântica não confirmada) | C | RO | Safe
- 0x60 Input Source | NC | RW | Dangerous (já existente) — values: 01 VGA-1, 03 DVI-1, 04 DVI-2, 0F DisplayPort-1, 10 DisplayPort-2, 11 HDMI-1, 12 HDMI-2
- 0x62 Audio Speaker Volume | C | RW | Safe (já existente)
- 0x6C/0x6E/0x70 Video Black Level Red/Green/Blue | C | RW | **Safe (novo)**
- 0x7E Trapezoid | C | RW | **Dangerous (novo)** — distorção de geometria; responde com falha neste monitor (retries excedidos — ver D-2026-09-26-full-osd-control-2), catalogado mesmo assim
- 0x87 Sharpness | C | RW | Safe (já existente)
- 0xAC Horizontal Frequency | C | RO | Safe
- 0xAE Vertical Frequency | C | RO | Safe
- 0xB2 Flat Panel Sub-Pixel Layout | NC | RO | Safe
- 0xB6 Display Technology Type | NC | RO | Safe
- 0xC6 (sem nome confiável) | C | RO | Safe
- 0xC8 Display Controller Type | NC | RO | Safe
- 0xC9 Display Firmware Level | C | RO | Safe
- 0xCA OSD/Button Control (lock) | NC | RW | Dangerous (já existente)
- 0xCC OSD Language | NC | RW | Safe (já existente) — values: 01 Chinese (traditional), 02 English, 03 French, 04 German, 06 Japanese, 0A Spanish, 0D Chinese (simplified)
- 0xD6 Power Mode | NC | RW | Dangerous (já existente) — values: 01 On, 04 Off (DPM), 05 Off (write-only)
- 0xDF VCP Version | C | RO | Safe
- 0xFD / 0xFF (sem nome confiável) | C | RO | Safe

**Invariante nova (trava para todo código, presente ou futuro):** um `access = ReadOnly` no catálogo NUNCA carrega `risk = Dangerous`. Motivo: `SoftwareOsd::set_feature` chama `authorize_write` (checa risco) ANTES de `validate_write` (checa acesso) — se um código RO fosse marcado Dangerous, um `set` sem `--yes` devolveria `DangerousWriteNotConfirmed` (exit 5, "repita com --yes") quando na verdade nenhum `--yes` jamais tornaria a escrita possível; o correto é `UnsupportedFeature` (exit 4). Todo código RO acima é `Safe` por isso, não porque escrevê-lo seja de fato inofensivo.

**Consolidação de `risk_for_code`:** a assinatura pública `risk_for_code(code: VcpCode) -> Risk` (usada por `authorize_write`) não muda, mas seu corpo passa a ler `catalog_entry(code).map(|e| e.risk).unwrap_or(Risk::Dangerous)` — absorve a tabela seed hardcoded de D-2026-09-25-core-domain-3 (cujas classificações Safe/Dangerous para os 16 códigos originais são preservadas byte a byte acima) numa única fonte de verdade; o fallback `Dangerous` para qualquer código fora do catálogo (inclui `>= 0xE0` manufacturer-specific) mantém o fail-safe de D-3 inalterado.

**`Capabilities::feature(code)`** passa a usar `catalog_entry(code).map(|e| e.kind)`/`.map(|e| e.access)` quando existir, com fallback ao comportamento atual (kind heurístico pelo caps, access sempre `ReadWrite`) para código fora do catálogo — implementa a nota de D-2026-09-25-core-domain-4 ("o catálogo dá kind independente do caps"). `Feature.allowed_values` continua vindo só do `Capabilities.vcp` (nunca do catálogo, campo inalterado) — é o dado real do monitor, não uma lista teórica; a fonte de fallback do catálogo entra só na VALIDAÇÃO (ver emenda abaixo), não no dado.

**Interpretação numérica** (funções puras em `mccs_catalog`, sem I/O): `hertz(raw: u16) -> f64 { f64::from(raw) / 100.0 }` para 0xAC/0xAE (confirma "144.00 Hz" a partir de `raw = 14400`); `version_from_u16(raw: u16) -> (u8, u8) { ((raw >> 8) as u8, (raw & 0xFF) as u8) }` para 0xC9/0xDF (confirma `0x0202 -> (2, 2)` = "2.2", medido em D-2026-09-25-ddc-backends-7). 0x0C/0x0B (temperatura de cor em Kelvin) NÃO ganham helper de conversão nesta phase — a fórmula VESA exata (`3000 + raw(0x0C) × raw(0x0B) × 100`) não foi confirmada com uma leitura real correlacionada dos dois códigos; expor o valor raw de 0x0C é suficiente para "controlar por nome", já que os presets nomeados de temperatura (5000/6500/7500/9300 K) já vêm resolvidos via 0x14. Registrado como todo, não bloqueia.

Alternativa rejeitada: duas tabelas separadas (uma de nome/kind/access, outra de risco, como D-3 originalmente fez) — descartada por DRY, cria risco de as duas divergirem quando o catálogo crescer numa phase futura.

---

**Emenda 2026-09-26 (pós-review da phase `cli`, carry-over W-5 de `.jdi/phases/cli/REVIEW.md`):** D-2026-09-26-cli-1 (já committed — max de escrita aprendido por UMA leitura quando desconhecido, "declarado no caps ou não") previu explicitamente que esta phase restringisse esse aprendizado a códigos `Continuous`; a review da `cli` confirmou o problema na letra atual do código: hoje, um `preset` (0x14) ou `OSD language` (0xCC) — ambos `NonContinuous`, `Safe` — com caps ausente ou ilegível aceita QUALQUER valor `<= max lido` **sem `--yes`**, o que é incompatível com a lista fechada que o MCCS define para um código discreto. Esta emenda é a decisão que D-2026-09-26-cli-1 pediu (registrada aqui, no arquivo do catálogo, e não num arquivo novo, porque só faz sentido justaposta à tabela acima). D-2026-09-26-cli-1 não é reescrita (append-only, já committed).

Três correções, todas compostas com o código já existente (sem novo parâmetro em `validate_write`, sem novo campo em `Feature`):

1. **`SoftwareOsd::max_for_write`** só chama `backend.read_vcp` (a leitura que aprende o max) quando `feature.kind == FeatureKind::Continuous`. Para `NonContinuous`/`Table` sem `allowed_values` do caps, devolve `Ok(None)` sem tocar o backend — restaura, para este caso, o "nunca escrever às cegas" no sentido mais estrito.
2. **`Feature::validate_write`** ganha um fallback: quando `self.allowed_values` (do caps) é `None`, passa a considerar os bytes de `mccs_catalog::catalog_entry(self.code).values` (quando o catálogo lista valores nomeados para o código — hoje: 0x14, 0x60, 0xCC, 0xD6, e os 4 códigos de reset de fábrica com o valor único `00 Reset` acrescentado à tabela acima) como a lista permitida, pelo mesmo caminho de erro `ValueNotAllowed` já existente. Um código `NonContinuous` sem lista do caps E sem valores no catálogo (0x02, 0x1E) cai no `known_max.ok_or(UnsupportedFeature)` de sempre — `max_for_write` já devolveu `None` (regra 1), então o resultado é `UnsupportedFeature`, igual ao comportamento anterior a D-2026-09-26-cli-1 (D-2026-09-25-core-domain-4) especificamente para esse caso.
   - **Sem a entrada `00 Reset`** nos 4 códigos de fábrica, o subcomando `reset` (D-2026-09-26-full-osd-control-4, que sempre escreve `0`) ficaria inoperante: são `WriteOnly`/`NonContinuous` sem `allowed_values` do caps, `max_for_write` nunca aprenderia um max (regra 1) e não haveria lista para validar — todo `reset` sairia `UnsupportedFeature`. Com a lista de um elemento, `set_feature` aceita exatamente `0` e recusa qualquer outro valor com `ValueNotAllowed`, mais correto que aceitar um `u16` arbitrário às cegas.
3. **`SoftwareOsd::set_feature`** passa a checar `feature.access == Access::ReadOnly || feature.kind == FeatureKind::Table` IMEDIATAMENTE após resolver a `Feature` (antes de `max_for_write`), devolvendo `UnsupportedFeature` sem nenhuma chamada ao backend — hoje a leitura de max roda antes de `validate_write` checar acesso/kind (`software_osd.rs:127-128` na iter 2 da `cli`), gastando uma leitura à toa; com o catálogo introduzindo ~15 códigos `ReadOnly`, isso deixa de ser hipotético (nota explícita de W-5). `Feature::validate_write` mantém sua própria checagem (inalterada, ainda a primeira coisa que faz) como defesa em profundidade para quem a chama direto (testes de `feature.rs`) — a mudança real em `set_feature` é só a ORDEM.
