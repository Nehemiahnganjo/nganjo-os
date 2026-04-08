#!/usr/bin/env bash
# Ng'anjo OS — Ng'anjo GUI Edition Chroot Customization Script
# Creator: Nehemiah Ng'anjo

TEAL='\033[38;2;0;210;180m'
RESET='\033[0m'
log() { echo -e "  ${TEAL}[chroot-neo]${RESET} $1"; }

# ── Locale ────────────────────────────────────────────────────────────────────
log "Generating locales..."
locale-gen

# ── Timezone ──────────────────────────────────────────────────────────────────
ln -sf /usr/share/zoneinfo/UTC /etc/localtime
hwclock --systohc 2>/dev/null || true
systemd-machine-id-setup 2>/dev/null || true

# ── Default shell ─────────────────────────────────────────────────────────────
chsh -s /bin/zsh root 2>/dev/null || true

# ── Services ─────────────────────────────────────────────────────────────────
log "Enabling services..."
systemctl enable NetworkManager
systemctl enable sddm
systemctl enable systemd-timesyncd
systemctl enable bluetooth
systemctl enable nganjo-cpu-performance
systemctl disable ModemManager 2>/dev/null || true
systemctl disable NetworkManager-wait-online 2>/dev/null || true
systemctl mask systemd-firstboot 2>/dev/null || true
systemctl mask ldconfig.service 2>/dev/null || true

# ── Live user ─────────────────────────────────────────────────────────────────
log "Creating live user 'nganjo'..."
useradd -m -G wheel,audio,video,storage,optical,network,input,kvm -s /bin/zsh nganjo 2>/dev/null || true
echo "nganjo:nganjo" | chpasswd
echo "root:nganjo"   | chpasswd
echo "%wheel ALL=(ALL:ALL) NOPASSWD: ALL" > /etc/sudoers.d/nganjo-live
chmod 440 /etc/sudoers.d/nganjo-live
mkdir -p /home/nganjo/{Desktop,Documents,Downloads,Music,Pictures,Videos}
chown -R nganjo:nganjo /home/nganjo

# ── SDDM autologin → Ng'anjo GUI session ──────────────────────────────────────
log "Configuring SDDM autologin into Ng'anjo GUI..."
mkdir -p /etc/sddm.conf.d
cat > /etc/sddm.conf.d/autologin.conf << 'EOF'
[Autologin]
User=nganjo
Session=nganjo-gui
EOF

# ── Plymouth boot theme ───────────────────────────────────────────────────────
log "Setting Plymouth theme..."
plymouth-set-default-theme -R nganjo 2>/dev/null || true

# ── Cleanup ───────────────────────────────────────────────────────────────────
log "Cleaning package cache..."
yes | pacman -Sc --noconfirm 2>/dev/null || true

log "Ng'anjo GUI chroot customization complete!"
