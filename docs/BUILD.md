# Build Guide

## Prerequisites

- Arch Linux host (or Arch-based distro)
- Root access
- Internet connection (for AUR packages during build)

## Install Build Dependencies

```bash
sudo pacman -S archiso squashfs-tools libisoburn dosfstools mtools
```

## Build the ISO

```bash
# Standard build
sudo bash scripts/build.sh

# Clean previous work/ directory first
sudo bash scripts/build.sh --clean

# Build and auto-test in QEMU
sudo bash scripts/build.sh --test
```

## Build Steps (what the script does)

| Step | Action |
|------|--------|
| 1 | Check build dependencies |
| 2 | Clean previous artifacts |
| 3 | Verify airootfs overlay |
| 4 | Apply branding (hostname, build date) |
| 5 | Pre-compile dconf database |
| 6 | Install chroot customization hook |
| 7 | Run mkarchiso |
| 8 | Generate SHA256 checksum |
| 9 | Report output path and size |

## Chroot Phase (network required)

During `mkarchiso`, the script runs `scripts/setup_chroot.sh` inside the chroot which:

- Generates locales and sets timezone
- Enables systemd services
- Creates the live user `nganjo`
- Installs `yay` AUR helper
- Builds and installs Calamares installer
- Installs AUR packages: `papirus-icon-theme`, `bibata-cursor-theme-bin`, `adw-gtk3`, GNOME extensions
- Sets Plymouth boot theme

> If no network is available, AUR packages are skipped. The ISO will still boot but the graphical installer may be missing.

## Output

```
out/
├── nganjo-os-1.0-lite-YYYY.MM.DD-x86_64.iso
└── nganjo-os-1.0-lite-YYYY.MM.DD-x86_64.iso.sha256
```

## Verify Checksum

```bash
sha256sum -c out/nganjo-os-*.iso.sha256
```
