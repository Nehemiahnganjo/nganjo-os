#!/usr/bin/env bash
# ══════════════════════════════════════════════════════════════════════════════
# Ng'anjo OS — Master ISO Build Script
# Creator: Nehemiah Ng'anjo
# Usage  : sudo bash scripts/build.sh [--clean] [--test]
# ══════════════════════════════════════════════════════════════════════════════

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
WORK_DIR="${REPO_ROOT}/work"
OUT_DIR="${REPO_ROOT}/out"
PROFILE_DIR="${REPO_ROOT}"
ISO_DATE=$(date +%Y.%m.%d)
ISO_NAME="nganjo-os-1.0-lite-${ISO_DATE}-x86_64.iso"

# ── Colors ─────────────────────────────────────────────────────────────────────
TEAL='\033[38;2;0;210;180m'
GOLD='\033[38;2;255;209;102m'
GREEN='\033[38;2;6;214;160m'
RED='\033[38;2;255;77;109m'
RESET='\033[0m'
BOLD='\033[1m'

log()  { echo -e "  ${TEAL}[✓]${RESET} $1"; }
warn() { echo -e "  ${GOLD}[!]${RESET} $1"; }
err()  { echo -e "  ${RED}[✗]${RESET} $1"; exit 1; }
step() { echo -e "\n  ${BOLD}${TEAL}▶ $1${RESET}\n"; }

# ── Root check ─────────────────────────────────────────────────────────────────
[[ $EUID -ne 0 ]] && err "Build script must run as root: sudo bash scripts/build.sh"

# ── Banner ─────────────────────────────────────────────────────────────────────
clear
echo -e "${TEAL}${BOLD}"
cat << 'EOF'
   ╔══════════════════════════════════════════════════════════════╗
   ║          Ng'anjo OS — ISO Build System                      ║
   ║          Creator: Nehemiah Ng'anjo                          ║
   ║          Version: 1.0 Lite — "Arise"                        ║
   ╚══════════════════════════════════════════════════════════════╝
EOF
echo -e "${RESET}"

# ── Parse args ─────────────────────────────────────────────────────────────────
DO_CLEAN=false
DO_TEST=false
EDITION="gnome"
for arg in "$@"; do
    case "$arg" in
        --clean) DO_CLEAN=true  ;;
        --test)  DO_TEST=true   ;;
        --kde)   EDITION="kde"  ;;
        --gnome) EDITION="gnome";;
        --tui)   EDITION="tui"  ;;
        --neo)   EDITION="neo"  ;;
    esac
done

# ── Step 1: Check Dependencies ────────────────────────────────────────────────
step "1/9 — Checking build dependencies"
DEPS=(archiso squashfs-tools libisoburn dosfstools mtools)
for dep in "${DEPS[@]}"; do
    if pacman -Qi "$dep" &>/dev/null; then
        log "${dep} — OK"
    else
        warn "${dep} not found — installing..."
        pacman -S --noconfirm "$dep" || err "Failed to install ${dep}"
    fi
done

# ── Step 2: Clean previous build ──────────────────────────────────────────────
step "2/9 — Cleaning previous build artifacts"
if [[ "$DO_CLEAN" == true ]] || [[ -d "$WORK_DIR" ]]; then
    rm -rf "$WORK_DIR" && log "work/ cleaned."
fi
mkdir -p "$OUT_DIR"
log "Output directory: ${OUT_DIR}"

# (stray directory cleanup removed — not needed)

# ── Step 3: Copy airootfs overlay ─────────────────────────────────────────────
step "3/9 — Verifying airootfs overlay"
[[ -d "${PROFILE_DIR}/airootfs" ]] || err "airootfs/ directory missing!"
log "airootfs overlay verified."

# ── Step 4: Apply branding ────────────────────────────────────────────────────
step "4/9 — Applying Ng'anjo OS branding"
# Update hostname
echo "nganjo-os" > "${PROFILE_DIR}/airootfs/etc/hostname"
log "Hostname set."

# Update os-release with current build date
sed -i "s/^BUILD_ID=.*/BUILD_ID=${ISO_DATE}/" \
    "${PROFILE_DIR}/airootfs/etc/os-release" 2>/dev/null || true
log "Build date stamped: ${ISO_DATE}"

# ── Step 5: Compile dconf database ────────────────────────────────────────────
step "5/9 — Preparing dconf database"
if [[ -d "${PROFILE_DIR}/airootfs/etc/dconf" ]]; then
    # Pre-compile dconf so it works offline without needing dconf update in chroot
    if command -v dconf &>/dev/null; then
        dconf compile "${PROFILE_DIR}/airootfs/etc/dconf/db/local" \
            "${PROFILE_DIR}/airootfs/etc/dconf/db/local.d" 2>/dev/null && \
            log "dconf database pre-compiled." || \
            log "dconf pre-compile skipped — will compile in chroot."
    else
        log "dconf not available on host — will compile in chroot."
    fi
else
    warn "dconf directory missing — GNOME defaults will not be applied."
fi

# ── Step 6: Prepare AUR packages hint ────────────────────────────────────────
step "6/9 — Wiring chroot customization hook"
if [[ "$EDITION" == "kde" ]]; then
    log "Edition: KDE Plasma (minimal)"
    cp "${PROFILE_DIR}/packages.kde.x86_64" "${PROFILE_DIR}/packages.x86_64.build"
    CHROOT_HOOK="${PROFILE_DIR}/airootfs/root/customize_airootfs.sh"
    cp "${SCRIPT_DIR}/setup_chroot_kde.sh" "$CHROOT_HOOK"
    ISO_NAME="nganjo-os-kde-1.0-lite-${ISO_DATE}-x86_64.iso"
elif [[ "$EDITION" == "tui" ]]; then
    log "Edition: TUI (terminal-only, no desktop)"
    cp "${PROFILE_DIR}/packages.tui.x86_64" "${PROFILE_DIR}/packages.x86_64.build"
    CHROOT_HOOK="${PROFILE_DIR}/airootfs/root/customize_airootfs.sh"
    cp "${SCRIPT_DIR}/setup_chroot_tui.sh" "$CHROOT_HOOK"
    ISO_NAME="nganjo-os-tui-1.0-lite-${ISO_DATE}-x86_64.iso"
else
    log "Edition: GNOME (default)"
    cp "${PROFILE_DIR}/packages.x86_64" "${PROFILE_DIR}/packages.x86_64.build"
    CHROOT_HOOK="${PROFILE_DIR}/airootfs/root/customize_airootfs.sh"
    cp "${SCRIPT_DIR}/setup_chroot.sh" "$CHROOT_HOOK"
    ISO_NAME="nganjo-os-gnome-1.0-lite-${ISO_DATE}-x86_64.iso"
fi
chmod +x "$CHROOT_HOOK"
# Swap the active package list so mkarchiso picks up the right one
cp "${PROFILE_DIR}/packages.x86_64.build" "${PROFILE_DIR}/packages.x86_64"
warn "AUR packages require network during build. Offline fallback is handled in the chroot script."

# ── Step 7: Build ISO ─────────────────────────────────────────────────────────
step "7/9 — Building ISO with mkarchiso"
echo -e "  This will take 20–45 minutes depending on your internet and hardware.\n"

mkarchiso \
    -v \
    -w "${WORK_DIR}" \
    -o "${OUT_DIR}" \
    "${PROFILE_DIR}" \
    && log "mkarchiso completed successfully." \
    || err "mkarchiso failed. Check the output above."

# ── Step 8: Generate checksum ─────────────────────────────────────────────────
step "8/9 — Generating SHA256 checksum"
ISO_PATH=$(find "${OUT_DIR}" -name "nganjo-os-*.iso" | head -1)
if [[ -f "$ISO_PATH" ]]; then
    sha256sum "$ISO_PATH" > "${ISO_PATH}.sha256"
    log "Checksum written: ${ISO_PATH}.sha256"
else
    warn "ISO file not found in output/ — skipping checksum."
fi

# ── Step 9: Report ────────────────────────────────────────────────────────────
step "9/9 — Build complete"
if [[ -f "$ISO_PATH" ]]; then
    ISO_SIZE=$(du -sh "$ISO_PATH" | cut -f1)
    echo -e "  ${GREEN}${BOLD}SUCCESS!${RESET}"
    echo -e "  ${TEAL}ISO     :${RESET} ${ISO_PATH}"
    echo -e "  ${TEAL}Size    :${RESET} ${ISO_SIZE}"
    echo -e "  ${TEAL}Date    :${RESET} ${ISO_DATE}"
    echo ""
    echo -e "  ${BOLD}Write to USB:${RESET}"
    echo -e "  ${TEAL}  sudo dd bs=4M if=\"${ISO_PATH}\" of=/dev/sdX status=progress oflag=sync${RESET}"
    echo ""
    echo -e "  ${BOLD}Test in QEMU:${RESET}"
    echo -e "  ${TEAL}  sudo bash scripts/test_iso.sh --uefi \"${ISO_PATH}\"${RESET}"
fi

# ── Optional: Auto-test ───────────────────────────────────────────────────────
if [[ "$DO_TEST" == true ]] && [[ -f "$ISO_PATH" ]]; then
    step "Bonus — Launching QEMU test"
    bash "${SCRIPT_DIR}/test_iso.sh" --uefi "$ISO_PATH"
fi
