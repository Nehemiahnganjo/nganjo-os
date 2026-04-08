#!/usr/bin/env bash
# Ng'anjo OS — TUI Edition Chroot Customization Script
# Creator: Nehemiah Ng'anjo

TEAL='\033[38;2;0;210;180m'
RESET='\033[0m'
log() { echo -e "  ${TEAL}[chroot-tui]${RESET} $1"; }

# ── Locale ────────────────────────────────────────────────────────────────────
log "Generating locales..."
locale-gen

# ── Timezone ──────────────────────────────────────────────────────────────────
ln -sf /usr/share/zoneinfo/UTC /etc/localtime
hwclock --systohc 2>/dev/null || true
systemd-machine-id-setup 2>/dev/null || true

# ── Default shell ─────────────────────────────────────────────────────────────
chsh -s /bin/zsh root 2>/dev/null || true

# ── Plymouth boot theme ───────────────────────────────────────────────────────
log "Setting Plymouth theme..."
plymouth-set-default-theme -R nganjo 2>/dev/null || true

# ── Services ─────────────────────────────────────────────────────────────────
log "Enabling services..."
systemctl enable NetworkManager
systemctl enable systemd-timesyncd
systemctl enable apparmor
systemctl enable ufw
systemctl disable ModemManager 2>/dev/null || true
systemctl disable NetworkManager-wait-online 2>/dev/null || true
systemctl mask systemd-firstboot 2>/dev/null || true
systemctl mask ldconfig.service 2>/dev/null || true

# ── TTY autologin for live session ────────────────────────────────────────────
log "Configuring TTY autologin..."
mkdir -p /etc/systemd/system/getty@tty1.service.d
cat > /etc/systemd/system/getty@tty1.service.d/autologin.conf << 'EOF'
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin nganjo --noclear %I $TERM
EOF

# ── Live user ─────────────────────────────────────────────────────────────────
log "Creating live user 'nganjo'..."
useradd -m -G wheel,audio,video,storage,optical,network,input -s /bin/zsh nganjo 2>/dev/null || true
echo "nganjo:nganjo" | chpasswd
echo "root:nganjo"   | chpasswd
echo "%wheel ALL=(ALL:ALL) NOPASSWD: ALL" > /etc/sudoers.d/nganjo-live
chmod 440 /etc/sudoers.d/nganjo-live
mkdir -p /home/nganjo/{Desktop,Documents,Downloads,Music,Pictures,Videos}
chown -R nganjo:nganjo /home/nganjo

# ── Auto-launch nganjo-tui on login ──────────────────────────────────────────
log "Wiring nganjo-tui auto-launch..."
cat >> /home/nganjo/.zshrc << 'ZSHEOF'

# Auto-launch Ng'anjo TUI on TTY1 login
if [[ -z "$DISPLAY" && -z "$WAYLAND_DISPLAY" && "$(tty)" == "/dev/tty1" ]]; then
    exec nganjo-tui
fi
ZSHEOF
chown nganjo:nganjo /home/nganjo/.zshrc

# Same for root
cat >> /root/.zshrc << 'ZSHEOF'

# Auto-launch Ng'anjo TUI on TTY1 login
if [[ -z "$DISPLAY" && -z "$WAYLAND_DISPLAY" && "$(tty)" == "/dev/tty1" ]]; then
    exec nganjo-tui
fi
ZSHEOF

# ── Cleanup ───────────────────────────────────────────────────────────────────
log "Cleaning package cache..."
yes | pacman -Sc --noconfirm 2>/dev/null || true

log "TUI chroot customization complete!"
