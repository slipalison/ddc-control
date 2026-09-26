# Linux DDC/CI setup

On Linux, `ddc-control` talks to monitors over DDC/CI through the kernel's
`i2c-dev` interface: one `/dev/i2c-*` device per I2C bus, including the
buses the GPU exposes for each display output. The setup below is done
once, by an administrator. The tool itself never needs root and never asks
for it.

## 1. Build dependency: libudev headers

The `ddc-hi` backend finds the display buses through udev, so building
`ddc-adapters` with its default `ddc-hi` feature needs the libudev
development files and `pkg-config`:

| Distribution  | Package                          |
|---------------|----------------------------------|
| Fedora / RHEL | `systemd-devel`                  |
| Debian/Ubuntu | `libudev-dev` and `pkg-config`   |

A build without the real backend (`--no-default-features`) does not need
them.

## 2. Load the `i2c-dev` module

```sh
sudo modprobe i2c-dev                                          # now
echo i2c-dev | sudo tee /etc/modules-load.d/i2c-dev.conf       # every boot
ls /dev/i2c-*                                                  # verify
```

Some distributions build `i2c-dev` into the kernel; then `/dev/i2c-*`
already exists and there is nothing to load.

## 3. Let your user open the display buses

Pick one of the two options.

### Option A: udev `uaccess` rule (recommended)

Grants read/write access to whoever is logged in at the local seat, and only
on buses provided by a display controller (PCI class `0x030000`). Other I2C
buses (sensors, SMBus, touchpads) stay root-only.

```sh
# /etc/udev/rules.d/60-ddc-control-i2c.rules
SUBSYSTEM=="i2c-dev", KERNEL=="i2c-[0-9]*", ATTRS{class}=="0x030000", TAG+="uaccess"
```

The `ddcutil` package ships the same rule as
`/usr/lib/udev/rules.d/60-ddcutil-i2c.rules`; if that file exists, you are
already set. Apply a new rule without rebooting:

```sh
sudo udevadm control --reload
sudo udevadm trigger --subsystem-match=i2c-dev
```

### Option B: an `i2c` group

```sh
sudo groupadd --system i2c
sudo usermod -aG i2c "$USER"
# /etc/udev/rules.d/60-ddc-control-i2c.rules
SUBSYSTEM=="i2c-dev", KERNEL=="i2c-[0-9]*", GROUP="i2c", MODE="0660"
```

Reload udev as above, then log out and back in so the new group applies.
This option opens every I2C bus to the group, not only the display ones.

### Verify

```sh
ls -l /dev/i2c-*        # Option A: a trailing '+' (ACL); Option B: group i2c, mode rw-rw----
getfacl /dev/i2c-5      # Option A: a "user:<you>:rw-" line; use your display's bus number
```

## 4. Never run it with `sudo`

Root is used once, for the setup above. `ddc-control` itself must never run
under `sudo` or as root. Root would open every I2C bus on the machine, not
only the display ones, to a process that has no reason to touch them. No
library or binary in this repository escalates privileges. If something
only works with `sudo`, the permissions are wrong: fix step 3 instead.

## 5. What to expect

- **Empty monitor list: check permissions first.** Buses the process
  cannot open, or whose EDID cannot be read, are skipped silently: `ddc-hi`
  logs a warning and moves on. An empty list almost always means step 2 or
  step 3 is missing.
- **A display with mute DDC/CI is still listed.** Enumeration only reads
  EDID and never probes DDC/CI. So a display with a readable EDID but no
  DDC/CI answer still shows up. Examples: most TVs, some docks and KVMs, and
  monitors with DDC/CI turned off in their OSD. Its reads and writes fail
  as a transport error once the 3 attempts are spent, in about 100 ms on
  the dev machine (a TV), without delaying calls to other monitors.
- **Timings.** On the dev machine, enumeration took about 1.1 s (an EDID
  read on each bus), a capabilities read about 2.5 s, and a VCP read or
  write 40–100 ms. The default budgets are 5 s, 8 s and 1 s. A failed
  transaction is retried up to 3 times within the same budget: 50 ms
  apart for VCP reads and writes, 500 ms apart for capabilities reads.
  A read or write on a monitor the backend has not listed yet
  first pays one enumeration, under the enumeration budget: about 1.2 s
  for the first read on a fresh backend.

## NVIDIA proprietary driver

The proprietary NVIDIA driver does not expose DDC/CI on every output. Some
GPU, connector and driver combinations, DisplayPort especially, show the
monitor missing from the list, or listed with every read failing as a
transport error or a timeout. This is a driver limitation, not a
`ddc-control` bug, and nothing crashes. `ddcutil`'s documentation describes
a commonly used workaround, a module option that switches the driver to
software I2C:

```sh
# /etc/modprobe.d/nvidia-i2c.conf
options nvidia NVreg_RegistryDwords=RMUseSwI2c=0x01;RMI2cSpeed=100
```

Rebuild the initramfs and reboot after adding it. If the monitor still does
not answer, try another output of the same GPU.
