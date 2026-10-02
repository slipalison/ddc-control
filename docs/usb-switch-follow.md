# Follow a USB switch (Linux)

A USB switch moves a keyboard and a mouse between two computers at the press
of a button; the monitor, plugged into both, stays on whatever input it was
on. With **Follow USB switch** on, `ddc-tray` closes that gap: when the
devices learned from the switch **leave** this computer, it switches the
monitor to the input of the other computer, over DDC/CI (VCP code `0x60`,
the input source).

The computer the switch leaves is the one that acts because it is the one on
screen at that moment: a monitor such as the dev machine's "RTK QHD HDR"
answers DDC/CI only on its active input, so the computer that is not on
screen cannot reach it.

This guide sets it up for two Linux computers sharing one monitor and one
switch — a **PC** and a **notebook** — each running `ddc-tray`. Do every step
on **both** machines.

## 1. DDC/CI access, on the PC and on the notebook

Each machine writes to the monitor itself, so each needs the `i2c-dev`
module and access to the `/dev/i2c-*` bus of its display. The deb and rpm
packages set both up; from a build of your own, follow
[`linux-ddc-setup.md`](linux-ddc-setup.md). In short:

```sh
sudo modprobe i2c-dev                                      # now
echo i2c-dev | sudo tee /etc/modules-load.d/i2c-dev.conf   # every boot (modules-load.d)
ls -l /dev/i2c-*                                           # the buses, with access for you
```

Then the udev rule of that guide (or the `i2c` group) lets your user open the
display buses. Never run `ddc-tray` with `sudo`.

Check it on each machine while that machine is on screen: `ddc-cli list`
shows the monitor and `ddc-cli get input` reads its current input. Note that
input: it is the one the **other** machine will switch to.

## 2. Learn the switch, on each machine

With the machine on screen and the keyboard and the mouse on it:

1. Open the tray menu and click **Learn USB switch**.
2. Within 30 seconds, press the switch's button.

Learning records the devices that left together and stayed away for about
1.5 s: the keyboard, the mouse, whatever else the switch carries. Hubs are
left out — the switch itself is usually one, and it stays. It also records
the monitor to switch: the one the tray's brightness entries act on, that is
the monitor the popup last loaded, else the first one listed. If nothing
leaves within the 30 seconds, nothing is recorded and the reason is printed
on stderr.

The last line of the follow's items in the menu shows what was learned, by
vendor and product id: `Learned: 046d:c077, 046d:c31c`, or `Learned: none`.
Two identical devices without a serial number count as one, and a wireless
receiver counts as one device; the follow only needs all of them to leave.

Learning never turns follow on and never writes to the monitor. Learning
again (to record another monitor, say) turns the follow off: turn it on
again afterwards.

## 3. Pick the target input, on each machine

The target input is the input of the **other** machine: where the monitor
must go when the switch leaves this one. The menu offers the standard MCCS
codes of `0x60`: **DisplayPort 1** (`0x0F`), **DisplayPort 2** (`0x10`),
**HDMI 1** (`0x11`) and **HDMI 2** (`0x12`). With the PC on DisplayPort 1
and the notebook on DisplayPort 2:

| Machine | Its own input | Pick, in its menu |
|---|---|---|
| PC | DisplayPort 1 (`0x0F`) | **DisplayPort 2** — when the switch leaves the PC, the monitor goes to `0x10` |
| notebook | DisplayPort 2 (`0x10`) | **DisplayPort 1** — when the switch leaves the notebook, the monitor goes to `0x0F` |

If your machines are on other inputs, pick accordingly: each machine picks
the input `ddc-cli get input` read on the other one.

## 4. Turn it on: what you consent to

Check **Follow USB switch** in the menu. It turns on only when the three
settings are there — learned devices, a monitor and a target input —;
otherwise it stays off and stderr says what is missing.

Switching the input source (`0x60`) is a Dangerous write for `ddc-control`:
it can leave the screen dark, so the popup and `ddc-cli` ask for a
confirmation each time. The follow cannot ask — you are no longer looking at
the machine the switch left —, so **turning the follow on is your consent**,
given once and kept while the follow is on, across restarts of the app. It
covers exactly one write: VCP code `0x60` on the monitor recorded by
learning, set to the target input you picked. Nothing else is written, to no
other monitor and no other code. Unchecking **Follow USB switch** withdraws
it.

## How it behaves

- Every 500 ms the follow reads the USB devices present
  (`/sys/bus/usb/devices`). When every learned device has been absent for 3
  reads in a row (about 1.5 s), it switches the monitor, once, and prints
  `ddc-tray: follow: switching <monitor-id> to input 0x10` (with the input
  picked) on stderr.
- It switches again only after at least one learned device came back. The
  devices arriving never switch anything, nor does a state found when the
  app starts, nor one device unplugged by hand while the others stay.
- **One try.** If the monitor does not answer — DDC/CI is off in its menu,
  or this machine is not on screen any more —, the error is printed on
  stderr (`ddc-tray: could not confirm the switch of …`) and that is all: no
  retry, no dialog. The switch may still have worked: once on the other
  input, the monitor stops answering this machine, so the read that confirms
  it fails.
- The write runs on a thread of its own: the follow keeps reading while the
  monitor settles on the new input (up to 3 s).

## The settings file

The settings live in `$XDG_CONFIG_HOME/ddc-control/usb-follow.json`, or
`~/.config/ddc-control/usb-follow.json` when `XDG_CONFIG_HOME` is not set:

```json
{
  "version": 1,
  "enabled": true,
  "devices": ["046d:c077", "046d:c31c:KB0001"],
  "monitor_id": "RTK-RTK-QHD-HDR-01010101",
  "target_input": 16
}
```

- `devices` are `vendor:product`, plus `:serial` when the device has one.
- `target_input` is in decimal: 16 is `0x10`, 15 is `0x0F`.
- The menu writes the file; there is no need to edit it. It is written
  through a temporary file and a rename, so it is never left half written.
- A file the app cannot read — another `version`, invalid JSON, an unknown
  input — is reported on stderr and left as it is, and the follow stays off
  until you learn or pick an input again, which rewrites it.

## Limits

- Linux and the tray only: not Windows or macOS, and not `ddc-cli`.
- One monitor and one switch.
- It acts when the devices leave, never when they arrive.
- It does not find out which input the other machine is on: you pick it.
- No retry of a failed write.
- It polls `/sys`; it does not listen to udev events.
