# Ng'anjo OS

**Version:** 1.0 Lite — "Arise"  
**By:** Nehemiah Ng'anjo  
**Base:** Arch Linux | GNOME on Wayland | x86_64

---

## What is this?

its my own linux distro built on arch. runs gnome on wayland, has a custom installer and works on most hardware out of the box. still in early stages but its functional.

---

## Features

- gnome on wayland (clean, no bloat)
- calamares installer (graphical)
- zsh with autosuggestions + syntax highlighting + fzf
- pipewire audio
- bluetooth works
- zram swap (good for low ram machines)
- plymouth boot animation
- cpu governor + io scheduler tuning
- firefox comes preinstalled, brave can be installed via aur
- flatpak ready

---

## Requirements

- 64bit cpu, at least 2 cores
- 2gb ram minimum (4gb recommended)
- 20gb disk space
- UEFI boot (no legacy bios support)

---

## How to Build

you need an arch linux machine to build this

```bash
# install archiso first
sudo pacman -S archiso

# then just run
sudo bash scripts/build.sh

# clean build
sudo bash scripts/build.sh --clean

# build and test in qemu
sudo bash scripts/build.sh --test
```

iso goes to `out/` folder when done. takes like 20-45 mins depending on your internet

---

## Testing

```bash
sudo bash scripts/test_iso.sh --uefi out/nganjo-os-*.iso
```

---

## Writing to USB

```bash
sudo dd bs=4M if=out/nganjo-os-*.iso of=/dev/sdX status=progress oflag=sync
```

change /dev/sdX to your actual usb drive. be careful not to wipe the wrong drive

---

## After Installing

run this after first boot:

```bash
sudo nganjo-setup
```

it will:
- update packages
- install yay
- setup ufw firewall
- enable apparmor
- fix mirrors with reflector
- add flathub

---

## Folder Structure

```
nganjo-os/
├── airootfs/       # all the os configs and scripts
├── docs/           # docs
├── efiboot/        # uefi boot stuff
├── grub/           # grub config
├── scripts/        # build and setup scripts
├── syslinux/       # legacy boot (just in case)
├── packages.x86_64 # package list
├── pacman.conf     # pacman config for build
└── profiledef.sh   # archiso profile
```

---

## Docs

- [Build Guide](docs/BUILD.md)
- [Install Guide](docs/INSTALL.md)
- [Changelog](docs/CHANGELOG.md)
- [Contributing](docs/CONTRIBUTING.md)

---

## License

GPL-2.0

---

*built for those who arise.*
