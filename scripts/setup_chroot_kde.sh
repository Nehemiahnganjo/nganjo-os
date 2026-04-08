#!/usr/bin/env bash
# Ng'anjo OS — KDE Edition Chroot Customization Script
# Creator: Nehemiah Ng'anjo

TEAL='\033[38;2;0;210;180m'
RESET='\033[0m'
log() { echo -e "  ${TEAL}[chroot-kde]${RESET} $1"; }

# ── Locale ────────────────────────────────────────────────────────────────────
log "Generating locales..."
locale-gen

# ── Timezone ──────────────────────────────────────────────────────────────────
ln -sf /usr/share/zoneinfo/UTC /etc/localtime
hwclock --systohc 2>/dev/null || true
systemd-machine-id-setup 2>/dev/null || true

# ── Default shell ─────────────────────────────────────────────────────────────
chsh -s /bin/zsh root 2>/dev/null || true

# ── Enable systemd services ───────────────────────────────────────────────────
log "Enabling services..."
systemctl enable NetworkManager
systemctl enable sddm
systemctl enable systemd-timesyncd
systemctl enable nganjo-cpu-performance
systemctl enable bluetooth
systemctl enable avahi-daemon
systemctl disable avahi-daemon 2>/dev/null || true
systemctl disable ModemManager 2>/dev/null || true
systemctl disable NetworkManager-wait-online 2>/dev/null || true
systemctl disable lvm2-monitor 2>/dev/null || true
systemctl mask systemd-firstboot 2>/dev/null || true
systemctl mask ldconfig.service 2>/dev/null || true

# ── Live user ─────────────────────────────────────────────────────────────────
log "Creating live user 'nganjo'..."
useradd -m -G wheel,audio,video,storage,optical,network,input,kvm -s /bin/zsh nganjo 2>/dev/null || true
echo "nganjo:nganjo" | chpasswd
echo "root:nganjo"   | chpasswd
echo "%wheel ALL=(ALL:ALL) NOPASSWD: ALL" > /etc/sudoers.d/nganjo-live
chmod 440 /etc/sudoers.d/nganjo-live
mkdir -p /home/nganjo/{Desktop,Documents,Downloads,Music,Pictures,Videos,Templates,Public}
chown -R nganjo:nganjo /home/nganjo

# ── SDDM auto-login for live session ─────────────────────────────────────────
log "Configuring SDDM auto-login..."
mkdir -p /etc/sddm.conf.d
cat > /etc/sddm.conf.d/autologin.conf << 'EOF'
[Autologin]
User=nganjo
Session=plasmawayland
EOF

# ── Plymouth boot theme ───────────────────────────────────────────────────────
log "Setting Plymouth theme..."
plymouth-set-default-theme -R nganjo 2>/dev/null || true

# ── KDE desktop file swap ─────────────────────────────────────────────────────
log "Swapping desktop files and icons for KDE..."

# Desktop shortcuts
cp /home/nganjo/Desktop/terminal.kde.desktop /home/nganjo/Desktop/terminal.desktop
cp /home/nganjo/Desktop/files.kde.desktop    /home/nganjo/Desktop/files.desktop
rm -f /home/nganjo/Desktop/*.kde.desktop

# Autostart
cp /home/nganjo/.config/autostart/nganjo-welcome.kde.desktop \
   /home/nganjo/.config/autostart/nganjo-welcome.desktop
rm -f /home/nganjo/.config/autostart/*.kde.desktop

# App .desktop files
cp /usr/share/applications/nganjo-installer.kde.desktop /usr/share/applications/nganjo-installer.desktop
cp /usr/share/applications/nganjo-welcome.kde.desktop   /usr/share/applications/nganjo-welcome.desktop
cp /usr/share/applications/nganjo-setup.kde.desktop     /usr/share/applications/nganjo-setup.desktop
rm -f /usr/share/applications/*.kde.desktop

# KDE uses the same PNG icons as GNOME — no swap needed

chown -R nganjo:nganjo /home/nganjo
gtk-update-icon-cache /usr/share/icons/hicolor 2>/dev/null || true

# ── Calamares — swap services config for KDE (sddm instead of gdm) ───────────
# (calamares removed — using nganjo-install terminal installer)

# ── AUR packages ─────────────────────────────────────────────────────────────
if ! curl -s --max-time 5 https://archlinux.org > /dev/null 2>&1; then
    log "No network — skipping AUR packages."
else
    log "Installing yay..."
    pacman -S --needed --noconfirm git base-devel
    cd /tmp
    git clone https://aur.archlinux.org/yay.git
    cd yay
    sudo -u nganjo HOME=/home/nganjo makepkg -si --noconfirm 2>/dev/null || \
        makepkg -si --noconfirm --asroot 2>/dev/null || true
    cd / && rm -rf /tmp/yay

    log "Installing AUR extras..."
    sudo -u nganjo yay -S --noconfirm --needed \
        papirus-icon-theme \
        bibata-cursor-theme-bin \
        2>/dev/null || log "Some AUR packages failed — non-fatal."
fi

# ── Copy KDE config to /etc/skel so installed users inherit it ───────────────
log "Installing KDE desktop config to skel..."
mkdir -p /etc/skel/.config /etc/skel/.local/share/color-schemes /etc/skel/.local/share/konsole

cp /home/nganjo/.config/kdeglobals            /etc/skel/.config/kdeglobals
cp /home/nganjo/.config/kwinrc                /etc/skel/.config/kwinrc
cp /home/nganjo/.config/plasmarc              /etc/skel/.config/plasmarc
cp /home/nganjo/.config/kscreenlockerrc       /etc/skel/.config/kscreenlockerrc
cp /home/nganjo/.config/konsolerc             /etc/skel/.config/konsolerc
cp /home/nganjo/.config/plasma-org.kde.plasma.desktop-appletsrc \
                                               /etc/skel/.config/plasma-org.kde.plasma.desktop-appletsrc
cp /home/nganjo/.local/share/color-schemes/NganjoOS.colors \
                                               /etc/skel/.local/share/color-schemes/NganjoOS.colors
cp /home/nganjo/.local/share/konsole/Nganjo.profile \
                                               /etc/skel/.local/share/konsole/Nganjo.profile
cp /home/nganjo/.local/share/konsole/NganjoOS.colorscheme \
                                               /etc/skel/.local/share/konsole/NganjoOS.colorscheme
log "KDE config installed to skel."

# ── Cleanup ───────────────────────────────────────────────────────────────────
log "Cleaning package cache..."
yes | pacman -Sc --noconfirm 2>/dev/null || true

log "KDE chroot customization complete!"
