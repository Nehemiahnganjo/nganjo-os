#!/usr/bin/env bash
# Ng'anjo OS — archiso Profile Definition
# Creator: Nehemiah Ng'anjo
# License: GPL-2.0

iso_name="nganjo-os"
iso_label="NGANJO_OS"
iso_publisher="Nehemiah Ng'anjo <nganjo@nganjo-os.org>"
iso_application="Ng'anjo OS Live/Rescue medium"
iso_version="1.0-lite"
install_dir="arch"
buildmodes=('iso')
bootmodes=(
    'bios.syslinux'
    'uefi.systemd-boot'
)
arch="x86_64"
pacman_conf="pacman.conf"
airootfs_image_type="squashfs"
airootfs_image_tool_options=('-comp' 'zstd' '-Xcompression-level' '19' '-b' '1M')
file_permissions=(
    ["/etc/shadow"]="0:0:400"
    ["/etc/passwd"]="0:0:644"
    ["/etc/group"]="0:0:644"
    ["/etc/sudoers.d"]="0:0:750"
    ["/etc/sudoers.d/nganjo-live"]="0:0:440"
    ["/etc/gdm/custom.conf"]="0:0:644"
    ["/etc/sddm.conf.d/autologin.conf"]="0:0:644"
    ["/root"]="0:0:750"
    ["/root/.automated_script.sh"]="0:0:755"
    ["/root/customize_airootfs.sh"]="0:0:755"
    ["/home/nganjo"]="1000:1000:755"
    ["/home/nganjo/Desktop/install.desktop"]="1000:1000:755"
    ["/home/nganjo/Desktop/terminal.desktop"]="1000:1000:755"
    ["/home/nganjo/.config/autostart/nganjo-welcome.desktop"]="1000:1000:644"
    ["/home/nganjo/.config/kdeglobals"]="1000:1000:644"
    ["/home/nganjo/.config/kwinrc"]="1000:1000:644"
    ["/home/nganjo/.config/plasmarc"]="1000:1000:644"
    ["/home/nganjo/.config/kscreenlockerrc"]="1000:1000:644"
    ["/home/nganjo/.config/konsolerc"]="1000:1000:644"
    ["/home/nganjo/.config/plasma-org.kde.plasma.desktop-appletsrc"]="1000:1000:644"
    ["/home/nganjo/.local/share/color-schemes/NganjoOS.colors"]="1000:1000:644"
    ["/home/nganjo/.local/share/konsole/Nganjo.profile"]="1000:1000:644"
    ["/home/nganjo/.local/share/konsole/NganjoOS.colorscheme"]="1000:1000:644"
    ["/usr/bin/nganjo-welcome"]="0:0:755"
    ["/usr/bin/nganjo-installer"]="0:0:755"
    ["/usr/bin/nganjo-install"]="0:0:755"
    ["/usr/bin/nganjo-setup"]="0:0:755"
    ["/usr/bin/nganjo-post-install"]="0:0:755"
    ["/usr/bin/nganjo-gui"]="0:0:755"
    ["/usr/share/nganjo"]="0:0:755"
    ["/usr/share/nganjo/logo.svg"]="0:0:644"
    ["/usr/share/nganjo/logo.png"]="0:0:644"
    ["/usr/share/nganjo/wallpaper-default.png"]="0:0:644"
    ["/usr/share/nganjo/gdm-background.png"]="0:0:644"
    ["/usr/share/icons/hicolor/256x256/apps/nganjo-installer.png"]="0:0:644"
    ["/usr/share/icons/hicolor/256x256/apps/nganjo-welcome.png"]="0:0:644"
    ["/usr/share/icons/hicolor/256x256/apps/nganjo-setup.png"]="0:0:644"
    ["/usr/share/icons/hicolor/256x256/apps/nganjo-installer-kde.svg"]="0:0:644"
    ["/usr/share/icons/hicolor/256x256/apps/nganjo-welcome-kde.svg"]="0:0:644"
    ["/usr/share/icons/hicolor/256x256/apps/nganjo-setup-kde.svg"]="0:0:644"
    ["/usr/share/applications/nganjo-installer.desktop"]="0:0:644"
    ["/usr/share/applications/nganjo-welcome.desktop"]="0:0:644"
    ["/usr/share/applications/nganjo-setup.desktop"]="0:0:644"
    ["/usr/share/plymouth/themes/nganjo"]="0:0:755"
    ["/usr/share/plymouth/themes/nganjo/nganjo.plymouth"]="0:0:644"
    ["/usr/share/plymouth/themes/nganjo/nganjo.script"]="0:0:644"
    ["/usr/share/plymouth/themes/nganjo/logo.png"]="0:0:644"
)
