# Hardware validation

The exact script to validate `ddc-cli` (phase `full-osd-control`, steps 0 to 5) and the tray app (phase `tray-app`, [its own section](#tray-app--phase-tray-app)) on the dev machine: Linux, the "RTK QHD HDR" monitor (`RTK-RTK-QHD-HDR-01010101`, `/dev/i2c-5`) and an LG TV whose DDC/CI is mute. Run it from the repository root, in order, one command at a time, and compare each result with the expected one.

Only reads and the few safe, reversible writes of step 3 and of the tray steps are allowed. Read [Never run](#never-run) and [Never do through the popup](#never-do-through-the-popup) before starting.

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
- Hardware tests: `7 passed`. `hardware_other_displays_fail_within_budget_and_worker_recovers` reads the LG in about 0.4 s (three attempts, 200 ms apart) and the RTK right after in about 45 ms. `hardware_reading_0x7e_never_stops_the_worker` prints `read_vcp 0x7E …: Err(Transport("ddc-hi panicked: index out of bounds: the len is 11 but the index is 11"))` and then a successful `read_vcp 0x10`. The panic message from the `ddc-hi-worker` thread on stderr is expected.

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

- `features`: exit 0, a header line and 28 rows, all with source `caps`; `jq length` prints `28`. `0x14` reads `1/11 sRGB`, `0x60` `15/3 DisplayPort-1`, `0xCA` `1/2 OSD disabled` (the RTK's own value; `ddcutil --bus 5 getvcp ca` names it the same), `0xCC` `2/13 English`, `0xD6` `1/5 On`; `0xFD` and `0xFF` read `not supported by this monitor`. About 3.7 s.
- `features --probe`: exit 0, 39 rows. The 11 with source `probe`, as the `jq` line prints them: `0x1E`, `0x20`, `0x30`, `0x62`, `0x6C`, `0x6E`, `0x70`, `0xC9`, `0xE6`, `0xF1` `ok`, and `0x7E` (126) `unresponsive`. `0x1E` reads `0/1 Off`. The Rust panic message for `0x7E` on stderr is expected. About 5 s.
  - Run the probe four times back to back: no row other than `0x7E` may read `unresponsive`. With VCP retries 200 ms apart (D-2026-09-26-full-osd-control-9), 4 of 4 runs were clean on 2026-09-26; at 50 ms, 3 of 4 had one code (most often `0x70`) failing all three attempts with `Expected DDC/CI length bit`. Record any failing code.
- `get preset`: `0x14 preset: 1 (0x01) sRGB, max 11 (0x0B)`.
- `get input`: `0x60 input: 15 (0x0F) DisplayPort-1, max 3 (0x03)`. About one read in ten, the RTK's firmware answers `16` or `17` with a valid checksum, where `ddcutil --bus 5 getvcp 60` reads `0x0F`; repeat the read, and record it if it happens.
- `get v-frequency`: the same raw value as `ddcutil --bus 5 getvcp AE --verbose`, run right after it; no particular number is expected. The RTK's reply depends on the video mode or the firmware's state: on 2026-09-26 both tools read `14400 (0x3840)`, `144.00 Hz`, at 2560x1600@144, and once both read `44818 (0xAF12)`, `448.18 Hz`, at 143.96 Hz. Record both readings and the display mode.
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

Optional, reads only: `$B -m LG get brightness` — exit 6 in about 3.6 s: the enumeration (~1.1 s), the TV's capabilities read failing three times 500 ms apart plus one presence check (~2.1 s), then the VCP read failing three times 200 ms apart (~0.4 s). `$B -m LG features --probe` — exit 0 after about 21 s; with the TV's capabilities unreadable, a warning, then all 39 catalogued codes `not responding`, about 0.4 s each.

## Never run

On any monitor, with or without `--yes`, never run a write to:

- the input source (`input`, `0x60`) or the power mode (`power`, `0xD6`);
- any factory reset (`reset factory`, `reset brightness-contrast`, `reset geometry`, `reset color`, or `0x04`, `0x05`, `0x06`, `0x08` by number);
- OSD control (`osd-lock`, `0xCA`), with any value: `osd-disabled` can leave the monitor without its own menu or buttons;
- auto setup (`auto-setup`, `0x1E`), with any value (`off`, `run`, `continuous`): it can move or resize the picture;
- geometry or trapezoid (`h-position` `0x20`, `v-position` `0x30`, `trapezoid` `0x7E`);
- the manufacturer-specific codes `0xE6` and `0xF1`, whose meaning is unknown;
- any code outside the [feature catalog](../README.md#feature-catalog), or any code written by number.

`set` accepts the catalog values of `0x1E` and `0xCA` with `--yes` (D-2026-09-26-full-osd-control-10), and that path is tested only against the in-memory backend; this script never exercises it on hardware. Reading any of these codes is fine.

## Tray app — phase `tray-app`

Same machine and monitors. The tray app is single-instance so that one process talks to the monitors: quit any running `ddc-tray` (**Quit** in its tray menu) before steps 6 and 7, and do not run `ddc-cli` while it is up.

### 6. Tray icon smoke test

Needs a desktop session whose tray is a StatusNotifierItem host (KDE Plasma, or GNOME with the AppIndicator extension) and `busctl`. Never `sudo`.

```sh
cargo build -p ddc-tray --release --locked
bash apps/ddc-tray/scripts/smoke-sni.sh --activate target/release/ddc-tray
bash apps/ddc-tray/scripts/smoke-sni.sh --fake --scroll target/release/ddc-tray
```

Expected: exit 0 each, a few seconds later, with

```text
smoke-sni: started target/release/ddc-tray as PID <pid>
smoke-sni: org.kde.StatusNotifierWatcher lists org.kde.StatusNotifierItem-<pid>-1/StatusNotifierItem, owned by PID <pid>
smoke-sni: called org.kde.StatusNotifierItem.Activate on org.kde.StatusNotifierItem-<pid>-1/StatusNotifierItem
smoke-sni: the app printed 'ddc-tray: popup shown' after Activate
smoke-sni: the popup was still shown 1.5 s later
smoke-sni: OK — PID <pid> registered its tray item, was alive 2 s later, showed its popup on Activate and kept it shown and never panicked
```

and

```text
smoke-sni: started target/release/ddc-tray as PID <pid> with DDC_TRAY_FAKE=1
smoke-sni: org.kde.StatusNotifierWatcher lists org.kde.StatusNotifierItem-<pid>-1/StatusNotifierItem, owned by PID <pid>
smoke-sni: the app serves the simulated monitor
smoke-sni: called org.kde.StatusNotifierItem.Scroll 120 Vertical on org.kde.StatusNotifierItem-<pid>-1/StatusNotifierItem
smoke-sni: the app printed 'ddc-tray: brightness 75 -> 80' after a vertical Scroll of +120
smoke-sni: called org.kde.StatusNotifierItem.Scroll 120 Horizontal on org.kde.StatusNotifierItem-<pid>-1/StatusNotifierItem
smoke-sni: a horizontal Scroll wrote nothing within 1.5 s
smoke-sni: called org.kde.StatusNotifierItem.Scroll -120 Vertical on org.kde.StatusNotifierItem-<pid>-1/StatusNotifierItem
smoke-sni: the app printed 'ddc-tray: brightness 80 -> 75' after a vertical Scroll of -120
smoke-sni: OK — PID <pid> registered its tray item, was alive 2 s later, stepped the brightness on a vertical Scroll only and never panicked
```

The script lists the watcher's items, starts the app with `DDC_TRAY_DEBUG=1`, and waits up to 15 s for a new item whose D-Bus connection belongs to the app's own PID (`GetConnectionUnixProcessID`); the app must then still be alive 2 s later, with no `panicked` on its stderr.

- `--activate` then calls `Activate` on that item, as the tray does on a left click, and requires `ddc-tray: popup shown` on the app's stderr within 5 s and no `ddc-tray: popup hidden` in the 1.5 s after it: the popup appears on screen for a moment. It only reads.
- `--fake` starts the app with `DDC_TRAY_FAKE=1`: a simulated RTK in memory (brightness 75 of 100), no real monitor touched, which the app must announce on stderr.
- `--scroll`, only with `--fake` (it writes, and the script refuses it otherwise), calls `Scroll` on the item as the host does for the wheel: one vertical notch (+120) must print `ddc-tray: brightness 75 -> 80` within 5 s, one horizontal notch no brightness line within 1.5 s, and one notch down (−120) `ddc-tray: brightness 80 -> 75`.
- Without options it stops after the registration checks. It always stops the app on the way out (SIGTERM, SIGKILL after 5 s). On Linux the app replaces itself at launch to turn WebKitGTK's DMA-BUF renderer off (D-2026-09-26-tray-app-10); the PID stays the same, so the check still holds.

Any other outcome is exit 1 with a `smoke-sni: FAIL:` line and the end of the app's stderr: no watcher on the session bus; the app exited before registering (most often another instance was already running); no item owned by that PID within 15 s; the app died within 2 s of registering; the item did not answer `Activate` or `Scroll` (an AppIndicator item, as the tray had until iteration 3, has no such method: `busctl` says "No such method", "Método inexistente" in Portuguese); no `popup shown` line within 5 s of `Activate`, or `popup hidden` within 1.5 s of it; no simulated-monitor notice with `--fake`; a missing brightness line after a vertical notch, or one after a horizontal notch; or `panicked` on its stderr.

The SIGTERM that stops the app quits it as **Quit** does, so its KWin placement script is unloaded (`popup placement unloaded from KWin` on its stderr). Only a SIGKILL or a crash leaves it loaded; it only matches the popup of that dead process, and the next start replaces it. To check: `busctl --user call org.kde.KWin /Scripting org.kde.kwin.Scripting isScriptLoaded s ddc-tray-anchor` (`b false` once no `ddc-tray` runs); to remove one by hand: the same call with `unloadScript`.

### 7. Tray hardware test

```sh
ddcutil --bus 5 getvcp 10
DDC_HW_TESTS=1 cargo test -p ddc-tray --locked -- --ignored rtk_qhd_hdr --test-threads=1 --nocapture
ddcutil --bus 5 getvcp 10
```

The command runs two tests. Both go through the app's own composition root (`compose_osd()`: the real backend with the capabilities cache), and neither confirms a write.

`rtk_qhd_hdr_panel_loads_and_one_safe_brightness_write_is_restored`:

1. It lists the monitors and picks the RTK by manufacturer and model.
2. It loads the RTK's panel, reads only: exactly `0x10 0x12 0x62 0x60 0x14 0xD6`, with 7 input sources, 7 color presets and 3 power modes.
3. It writes brightness once: the original value +10, or −10 when +10 would pass the maximum, and expects that value read back.
4. A guard writes the original value back, also when an assertion failed first, and expects both the value read back and a fresh read to equal it.

`rtk_qhd_hdr_mute_monitor_fails_its_panel_and_the_rtk_loads`, reads only:

1. It lists the monitors and expects exactly one RTK, plus at least one other monitor (the LG TV).
2. It loads the panel of every other monitor and expects it to fail with `transport` or `timeout`, as the popup needs to skip a mute monitor.
3. It loads the RTK's panel and expects the six quick controls.

Expected: `2 passed`, with the timings, the monitors, the panel, `brightness <original> -> <target> (max 100): read back Ok(ReadBackDto { current: <target>, max: 100 })`, `restored brightness: Ok(ReadBackDto { current: <original>, max: 100 })`, `LG TV SSCR2: Err(UiError { kind: Transport, message: "transport error: DDC/CI I2C error: Input/output error (os error 5) (gave up after attempt 3 of 3)" })` and `RTK panel codes: [10, 12, 62, 60, 14, D6]`; both `ddcutil` reads show the same brightness. On 2026-09-26: `list_monitors` 1.17 s, `load_panel` 0.51 s, the write and the restore about 146 ms each, brightness 100 → 90 → 100. On 2026-09-27 the same, and the TV's `load_panel` failed in 4.83 s, the RTK's loaded in 0.76 s. Without `DDC_HW_TESTS=1` the tests print `skipped: …` and pass without touching a monitor. With `DDC_TRAY_FAKE` set, to any value, both fail at once with `DDC_TRAY_FAKE is set: …`, before any monitor is opened: the simulated monitor has the RTK's id and model, and a run on it would be no evidence of the real one. If the second `ddcutil` read differs from the first, put the value back with `$B -m RTK set brightness <original>` (step 0 builds `$B`).

### 8. The popup by hand (optional)

Start `target/release/ddc-tray` and left-click its tray icon. On the dev machine the LG TV is listed first, and its DDC/CI is mute, so its panel fails: check that the popup opens on the RTK (after about 5 s the first time, while it tries the TV) and that the selector at the top lists the TV as "LG TV SSCR2 (no DDC/CI)". On KDE Plasma under Wayland, check that it opens next to the icon, not in the middle of the screen, and that it has no taskbar entry. Then only:

- left-click the icon again: the popup hides; click it once more: it shows;
- open the monitor selector and the color preset list, and check that the popup stays open while the list is open and after a pick (the bug of iteration 3); Esc on an open list closes only the list;
- move brightness, contrast and volume and change the color preset, putting each back to its step 0 value right after;
- roll the mouse wheel over the icon one notch at a time while the popup shows the RTK: brightness moves 5% per notch and the open popup follows; put brightness back;
- use the menu's **Brightness** entries (right click) only while the popup shows the RTK (they set the monitor the popup last loaded), then put brightness back;
- open the input and power dialogs only to read them, and close them with **Cancel** or Esc;
- open **All settings** and run **Probe hidden settings**, which only read, without changing any entry.

Choose **Quit** in the tray menu when done, and record anything the popup shows that `ddc-cli` reads differently.

### Never do through the popup

On any monitor, never press **Apply** in the confirmation dialog, which means never completing:

- an input switch (the input chips): the screen may go dark, and only the monitor's own buttons bring it back;
- a power mode change (the power button in the header): the monitor may turn off;
- a change to any entry tagged **Caution** under **All settings** or among the probed ones: OSD control `0xCA`, auto setup `0x1E`, geometry `0x20`, `0x30` and `0x7E`, the manufacturer-specific `0xE6` and `0xF1`, or any other code the core marks dangerous — the codes of [Never run](#never-run). A **Caution** slider opens the dialog when released: cancel it.

The tray's hardware test never confirms a write, and neither does anyone validating the tray by hand.
