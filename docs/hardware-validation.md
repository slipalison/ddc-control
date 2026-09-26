# Hardware validation — phase `full-osd-control`

The exact script to validate `ddc-cli` on the dev machine: Linux, the "RTK QHD HDR" monitor (`RTK-RTK-QHD-HDR-01010101`, `/dev/i2c-5`) and an LG TV whose DDC/CI is mute. Run it from the repository root, in order, one command at a time, and compare each result with the expected one.

Only reads and the few safe, reversible writes of step 3 are allowed. Read [Never run](#never-run) before starting.

## 0. Build and record the original values

```sh
cargo build --release -p ddc-cli --locked
B=target/release/ddc-cli
```

Leave at least 6 s between two commands that read capabilities from the monitor (`caps --refresh`, or the first command after the cache directory is cleared); the RTK refuses a capabilities read issued right after another one.

Record the values the writes of step 3 change, and keep the output:

```sh
$B -m RTK get brightness --json
$B -m RTK get contrast --json
$B -m RTK get preset --json
$B -m RTK get volume --json
BRIGHTNESS0=$($B -m RTK get brightness --json | jq .current)
CONTRAST0=$($B -m RTK get contrast --json | jq .current)
PRESET0=$($B -m RTK get preset --json | jq .current)
VOLUME0=$($B -m RTK get volume --json | jq .current)
echo "$BRIGHTNESS0 $CONTRAST0 $PRESET0 $VOLUME0"
```

Expected: exit 0 each; on 2026-09-26 the RTK read brightness 100, contrast 50, preset 1 (`"value_name": "sRGB"`), volume 30. `get volume` also warns on stderr that `0x62` is not declared in the capabilities.

## 1. Monitors, capabilities and the hardware tests

```sh
$B list
$B -m RTK caps
DDC_HW_TESTS=1 cargo test -p ddc-adapters --locked --test real_monitor -- --ignored --test-threads=1 --nocapture
```

Expected:

- `list`: exit 0, two lines, `GSM-LG-TV-SSCR2-01010101` and `RTK-RTK-QHD-HDR-01010101`.
- `caps`: exit 0, `mccs version: 2.2`, 28 VCP codes, `0x14 preset: 0x01 0x02 0x04 0x05 0x06 0x08 0x0B`.
- Hardware tests: `7 passed`, including `hardware_reading_0x7e_never_stops_the_worker`, which prints `read_vcp 0x7E …: Err(Transport("ddc-hi panicked: index out of bounds: the len is 11 but the index is 11"))` and then a successful `read_vcp 0x10`. The panic message from the `ddc-hi-worker` thread on stderr is expected.

## 2. Features and named values (reads only)

```sh
time $B -m RTK features
$B -m RTK features --json | jq length
time $B -m RTK features --probe
$B -m RTK features --probe --json | jq -c '.[] | select(.declared_in_capabilities == false) | [.code, .probe_status]'
$B -m RTK get preset
$B -m RTK get input
$B -m RTK get v-frequency
$B -m RTK get vcp-version
```

Expected:

- `features`: exit 0, a header line and 28 rows, all with source `caps`; `jq length` prints `28`. `0x14` reads `1/11 sRGB`, `0x60` `15/3 DisplayPort-1`, `0xCC` `2/13 English`, `0xD6` `1/5 On`; `0xFD` and `0xFF` read `not supported by this monitor`. About 3.7 s.
- `features --probe`: exit 0, 39 rows. The 11 with source `probe`, as the `jq` line prints them: `0x1E`, `0x20`, `0x30`, `0x62`, `0x6C`, `0x6E`, `0x70`, `0xC9`, `0xE6`, `0xF1` `ok`, and `0x7E` (126) `unresponsive`. The Rust panic message for `0x7E` on stderr is expected. About 5 s.
  - On the RTK, back-to-back reads now and then fail three attempts in a row (`Expected DDC/CI length bit`), so one more row may read `not responding` / `unresponsive` (seen most on `0x70`). Run the probe up to three times: it passes if every one of those 10 codes reads `ok` in at least one run and `0x7E` is `unresponsive` in every run. Record how many runs it took.
- `get preset`: `0x14 preset: 1 (0x01) sRGB, max 11 (0x0B)`.
- `get input`: `0x60 input: 15 (0x0F) DisplayPort-1, max 3 (0x03)`. Rarely the RTK answers `16` or `17` with a valid checksum; repeat the read, and record it if it happens.
- `get v-frequency`: the same raw value as `ddcutil --bus 5 getvcp AE --verbose`. On 2026-09-26, at 143.96 Hz, both read `44818 (0xAF12)`, shown as `448.18 Hz` — the RTK's own reply, not 144.00 Hz. Record what it reads.
- `get vcp-version`: `0xDF vcp-version: 514 (0x202) 2.2, max 65535 (0xFFFF)`.

## 3. Safe writes, each undone right away

Only these writes, in this order. Each must exit 0 and, with `--json`, report `"applied": true`; restore the original value before moving on.

```sh
$B -m RTK set brightness 80 --json
$B -m RTK set brightness "$BRIGHTNESS0" --json

$B -m RTK set contrast 55 --json
$B -m RTK set contrast "$CONTRAST0" --json

$B -m RTK set preset 6500k --json
$B -m RTK get preset
$B -m RTK set preset srgb --json

$B -m RTK set volume 35 --json
$B -m RTK set volume "$VOLUME0" --json
```

Expected:

- `set brightness 80`: `"current": 80`, `"applied": true`; the screen dims visibly. The restore reads back the original value.
- `set contrast 55`: `"current": 55`, `"applied": true`.
- `set preset 6500k`: writes `0x05`, `"current": 5`, `"value_name": "6500 K"`, `"applied": true`; `get preset` then reads `5 (0x05) 6500 K`. `set preset srgb` writes `0x01` and reads back `"value_name": "sRGB"`. With HDR enabled on the display, the monitor may keep its preset: then `"applied": false` with a warning, still exit 0 — record it.
- `set volume 35`: the undeclared `0x62` is written after one read of its maximum (D-2026-09-26-cli-1), `"current": 35`, `"applied": true`; a warning says `0x62` is not declared.
- If a read-back fails (exit 6) or looks wrong, the write may still have been applied: check with `$B -m RTK get <name>` before retrying.

## 4. Refusals that write nothing

```sh
$B -m RTK set preset 3; echo "exit $?"
$B -m RTK set brightness srgb; echo "exit $?"
$B -m RTK get preset
$B -m RTK get brightness
```

Expected:

- `set preset 3`: exit 4, `value 3 is not an allowed value for feature 0x14` — `0x03` is not in the RTK's preset list, so nothing is written.
- `set brightness srgb`: exit 2, `0x10 brightness has no value named 'srgb'; give it a number`, before any monitor is touched.
- `get preset` and `get brightness` still read the original values.

## 5. Final state

```sh
$B -m RTK get brightness --json
$B -m RTK get contrast --json
$B -m RTK get preset --json
$B -m RTK get volume --json
```

Expected: `current` equals `$BRIGHTNESS0`, `$CONTRAST0`, `$PRESET0` and `$VOLUME0`.

Optional, reads only: `$B -m LG features --probe` — exit 0 after about 9 s; with the TV's capabilities unreadable, a warning, then all 39 catalogued codes `not responding`.

## Never run

On any monitor, with or without `--yes`, never run a write to:

- the input source (`input`, `0x60`) or the power mode (`power`, `0xD6`);
- any factory reset (`reset factory`, `reset brightness-contrast`, `reset geometry`, `reset color`, or `0x04`, `0x05`, `0x06`, `0x08` by number);
- the OSD lock (`osd-lock`, `0xCA`);
- auto setup, geometry or trapezoid (`auto-setup` `0x1E`, `h-position` `0x20`, `v-position` `0x30`, `trapezoid` `0x7E`);
- the manufacturer-specific codes `0xE6` and `0xF1`, whose meaning is unknown;
- any code outside the [feature catalog](../README.md#feature-catalog), or any code written by number.

Reading any of them is fine.
