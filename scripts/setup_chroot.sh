#!/usr/bin/env bash
# Ng'anjo OS — Chroot Customization Script
# Creator: Nehemiah Ng'anjo
# Runs inside the archiso chroot to install AUR packages and finalize branding

TEAL='\033[38;2;0;210;180m'
RESET='\033[0m'
log() { echo -e "  ${TEAL}[chroot]${RESET} $1"; }

# ── Locale ────────────────────────────────────────────────────────────────────
log "Generating locales..."
locale-gen
log "Locales generated."

# ── Timezone — set UTC as default, NTP auto-syncs to local time ───────────────
log "Setting default timezone to UTC (NTP will auto-detect)..."
ln -sf /usr/share/zoneinfo/UTC /etc/localtime
hwclock --systohc 2>/dev/null || true
systemd-machine-id-setup 2>/dev/null || true
log "Timezone and NTP configured."

# ── Default shell ─────────────────────────────────────────────────────────────
log "Setting default shell to zsh..."
chsh -s /bin/zsh root 2>/dev/null || true

# ── dconf database ────────────────────────────────────────────────────────────
log "Compiling dconf database..."
dconf update 2>/dev/null || true

# ── Enable systemd services ───────────────────────────────────────────────────
log "Enabling systemd services..."
systemctl enable NetworkManager
systemctl enable gdm
systemctl enable systemd-timesyncd
systemctl enable nganjo-cpu-performance
# bluetooth/avahi enabled but not started — saves ~15MB RAM at idle
systemctl enable bluetooth
systemctl enable avahi-daemon
systemctl disable avahi-daemon 2>/dev/null || true
# Boot speed — disable slow/unused services
systemctl disable ModemManager 2>/dev/null || true
systemctl disable NetworkManager-wait-online 2>/dev/null || true
systemctl disable systemd-networkd-wait-online 2>/dev/null || true
systemctl disable lvm2-monitor 2>/dev/null || true
systemctl disable mdadm 2>/dev/null || true
systemctl disable remote-fs.target 2>/dev/null || true
systemctl mask systemd-firstboot 2>/dev/null || true
systemctl mask ldconfig.service 2>/dev/null || true
log "Services enabled."

# ── Live user ─────────────────────────────────────────────────────────────────
log "Creating live user 'nganjo'..."
useradd -m -G wheel,audio,video,storage,optical,network,input,kvm -s /bin/zsh nganjo 2>/dev/null || true
echo "nganjo:nganjo" | chpasswd
echo "root:nganjo"   | chpasswd
echo "%wheel ALL=(ALL:ALL) NOPASSWD: ALL" > /etc/sudoers.d/nganjo-live
chmod 440 /etc/sudoers.d/nganjo-live

# ── Create XDG user directories ───────────────────────────────────────────────
mkdir -p /home/nganjo/{Desktop,Documents,Downloads,Music,Pictures,Videos,Templates,Public}
chown -R nganjo:nganjo /home/nganjo
log "Live user created with home directories."

# ── GDM auto-login for live session ───────────────────────────────────────────
mkdir -p /etc/gdm
cat > /etc/gdm/custom.conf << 'GDMCONF'
[daemon]
AutomaticLoginEnable=True
AutomaticLogin=nganjo
TimedLoginEnable=False

[security]

[xdmcp]

[chooser]

[debug]
GDMCONF
log "GDM auto-login configured."

# ── Install yay from AUR (for building AUR packages) ─────────────────────────
log "Checking network connectivity..."
if ! curl -s --max-time 5 https://archlinux.org > /dev/null 2>&1; then
    log "No network — skipping AUR packages (yay, papirus, bibata, extensions)."
    log "Run setup_chroot.sh manually after connecting to the internet."
else
    log "Installing yay AUR helper..."
    pacman -S --needed --noconfirm git base-devel
    cd /tmp
    git clone https://aur.archlinux.org/yay.git
    cd yay
    sudo -u nganjo HOME=/home/nganjo makepkg -si --noconfirm 2>/dev/null || \
        makepkg -si --noconfirm --asroot 2>/dev/null || true
    cd /
    rm -rf /tmp/yay
    log "yay installed."

    log "Installing AUR branding packages..."
    export HOME=/home/nganjo
    sudo -u nganjo yay -S --noconfirm --needed brave-bin 2>/dev/null && \
        log "brave-bin installed." || log "brave-bin failed — firefox is the fallback."

    sudo -u nganjo yay -S --noconfirm --needed \
        papirus-icon-theme \
        bibata-cursor-theme-bin \
        adw-gtk3 \
        gnome-shell-extension-dash-to-dock \
        gnome-shell-extension-appindicator \
        gnome-shell-extension-blur-my-shell \
        2>/dev/null || log "Some AUR packages failed — non-fatal, continuing."
    log "AUR packages done."

    sudo -u nganjo gsettings set org.gnome.desktop.interface icon-theme 'Papirus-Dark' 2>/dev/null || true
    sudo -u nganjo gsettings set org.gnome.desktop.interface cursor-theme 'Bibata-Modern-Ice' 2>/dev/null || true
fi

# ── Remove default GNOME wallpapers ──────────────────────────────────────────
log "Removing default GNOME wallpapers..."
rm -rf /usr/share/backgrounds/gnome 2>/dev/null || true
rm -rf /usr/share/gnome-background-properties 2>/dev/null || true
log "Default wallpapers removed."

# ── Plymouth boot theme ───────────────────────────────────────────────────────
log "Setting Plymouth theme..."
plymouth-set-default-theme -R nganjo 2>/dev/null || true
log "Plymouth theme set."
log "Patching hicolor index.theme..."
THEME=/usr/share/icons/hicolor/index.theme
for size in 16x16 32x32 48x48 64x64 128x128 256x256; do
    mkdir -p "/usr/share/icons/hicolor/${size}/apps"
    grep -q "^Directories=" "$THEME" && \
        sed -i "s|^Directories=.*|&,${size}/apps|" "$THEME" || true
    if ! grep -q "^\[${size}/apps\]" "$THEME"; then
        printf "\n[%s/apps]\nSize=%s\nType=Fixed\n" "$size" "${size%%x*}" >> "$THEME"
    fi
done
log "hicolor index.theme patched."

# ── Cleanup ───────────────────────────────────────────────────────────────────
log "Cleaning package cache..."
yes | pacman -Sc --noconfirm 2>/dev/null || true
command -v yay &>/dev/null && sudo -u nganjo yay -Sc --noconfirm 2>/dev/null || true

log "Chroot customization complete!"
