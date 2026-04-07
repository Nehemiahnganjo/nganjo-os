# Ng'anjo OS

**Version:** 1.0 Lite — *"Arise"*  
**Creator:** Nehemiah Ng'anjo  
**Base:** Arch Linux | **Desktop:** GNOME on Wayland | **Arch:** x86_64

---

## Overview

Ng'anjo OS is a lightweight, performance-focused Arch Linux-based distribution featuring a clean GNOME/Wayland desktop, a custom installer, and out-of-the-box hardware support.

---

## Features

- GNOME on Wayland with custom branding and theming
- Graphical installer via Calamares
- zsh with autosuggestions, syntax highlighting, and fzf
- PipeWire audio, Bluetooth, NetworkManager
- zram swap for improved performance on low-RAM systems
- Plymouth boot animation
- Custom performance tuning (CPU governor, I/O scheduler, sysctl)
- Firefox pre-installed; Brave available via AUR
- Flatpak + Flathub ready

---

## Requirements

| Component | Minimum |
|-----------|---------|
| CPU | x86_64, 2 cores |
| RAM | 2 GB (4 GB recommended) |
| Disk | 20 GB |
| Boot | UEFI |

---

## Building the ISO

```bash
# Install dependencies (Arch Linux host required)
sudo pacman -S archiso

# Build
sudo bash scripts/build.sh

# Build + clean previous artifacts
sudo bash scripts/build.sh --clean

# Build + auto-launch QEMU test
sudo bash scripts/build.sh --test
```

Output ISO is written to `out/`.

---

## Testing in QEMU

```bash
sudo bash scripts/test_iso.sh --uefi out/nganjo-os-*.iso
```

---

## Writing to USB

```bash
sudo dd bs=4M if=out/nganjo-os-*.iso of=/dev/sdX status=progress oflag=sync
```

Replace `/dev/sdX` with your USB device.

---

## Post-Install Setup

After installing to disk, run:

```bash
sudo nganjo-setup
```

This will:
- Update system packages
- Install yay (AUR helper)
- Enable UFW firewall and AppArmor
- Optimize pacman mirrors with reflector
- Enable Flatpak + Flathub

---

## Project Structure

```
nganjo-os/
├── airootfs/          # OS overlay (configs, scripts, branding)
├── docs/              # Additional documentation
├── efiboot/           # UEFI systemd-boot entries
├── grub/              # GRUB boot config
├── scripts/           # Build, chroot, post-install, test scripts
├── syslinux/          # Legacy BIOS boot
├── packages.x86_64    # Package list
├── pacman.conf        # Pacman config used during build
└── profiledef.sh      # archiso profile definition
```

---

## Documentation

- [Build Guide](docs/BUILD.md)
- [Install Guide](docs/INSTALL.md)
- [Changelog](docs/CHANGELOG.md)
- [Contributing](docs/CONTRIBUTING.md)

---

## License

GPL-2.0 — see [LICENSE](LICENSE)

---

*Ng'anjo OS — Built for those who arise.*
