# Install Guide

## boot the iso

write it to usb first:

```bash
sudo dd bs=4M if=out/nganjo-os-*.iso of=/dev/sdX status=progress oflag=sync
```

make sure uefi is enabled in bios. legacy bios wont work.

boot from usb and it will auto login to the live session as user `nganjo`.

## installing

click the install icon on the desktop or run `nganjo-installer` in terminal.

follow the steps in calamares:
- pick language
- pick keyboard
- partition your disk
- create your user
- hit install

reboot when its done.

## after install

first thing to do after booting into your installed system:

```bash
sudo nganjo-setup
```

this sets up the firewall, installs yay, fixes mirrors, adds flathub etc.

## live session login

| user | password |
|------|----------|
| nganjo | nganjo |
| root | nganjo |

these are only for the live session. you set your own password during install.

## partitioning

if you're doing manual partitioning:

- efi partition: 512mb, fat32
- root `/`: 20gb+, ext4 or btrfs
- no swap needed, zram handles that automatically

## common issues

**black screen on boot** — use the VM/fallback boot entry in grub, it adds nomodeset

**calamares missing** — connect to internet and run `yay -S calamares`

**no sound** — run:
```bash
systemctl --user enable --now pipewire pipewire-pulse wireplumber
```
