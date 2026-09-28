#!/bin/sh
# Post-install script of the ddc-control deb and rpm packages
# (D-2026-09-28-release-packaging-4). Loads i2c-dev now and applies the
# packaged udev rule without a reboot; at the next boot, modules-load.d and
# udev do the same on their own. Best effort: a container or a chroot has no
# udev and no kernel module, and nothing here may fail the installation.
modprobe i2c-dev 2>/dev/null || true
udevadm control --reload 2>/dev/null || true
udevadm trigger --subsystem-match=i2c-dev 2>/dev/null || true
exit 0
