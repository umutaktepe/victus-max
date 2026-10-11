#!/usr/bin/env bash
# ==============================================================================
# Victus Max System Updater Helper
# Invoked with root privileges (typically via pkexec) from the GUI or CLI.
# Emits [STAGE:<TAG>] markers for real-time GUI progress tracking.
# ==============================================================================
set -euo pipefail

if [ "$EUID" -ne 0 ]; then
    echo "ERROR: victus-max-updater must be run as root." >&2
    exit 1
fi

if [ -z "${SUDO_USER:-}" ] && [ -n "${PKEXEC_UID:-}" ]; then
    export SUDO_USER="$(id -nu "$PKEXEC_UID" 2>/dev/null || true)"
fi
export VICTUS_MAX_GUI_UPDATE=1

CHANNEL="${1:-canary}"
REPO_URL="https://github.com/umutaktepe/victus-max.git"
WORK_DIR="/tmp/victus-max-update"

echo "[STAGE:PREPARE] Sistem bağımlılıkları ve çalışma ortamı hazırlanıyor..."

cleanup() {
    rm -rf "$WORK_DIR"
}
trap cleanup EXIT

echo "[STAGE:DOWNLOAD] GitHub üzerinden en son kaynak kodlar alınıyor (${CHANNEL})..."
rm -rf "$WORK_DIR"
mkdir -p "$WORK_DIR"

if command -v git &>/dev/null; then
    if [ "$CHANNEL" == "stable" ]; then
        # Check if tags exist (sorted by semantic version)
        LATEST_TAG=$(git ls-remote --tags --refs --sort='v:refname' "$REPO_URL" | tail -n1 | awk '{print $2}' | sed 's|refs/tags/||' || true)
        if [ -n "$LATEST_TAG" ]; then
            echo "Latest release tag found: $LATEST_TAG"
            git clone --depth 1 -b "$LATEST_TAG" "$REPO_URL" "$WORK_DIR"
        else
            echo "No tags found. Falling back to main branch..."
            git clone --depth 1 -b main "$REPO_URL" "$WORK_DIR"
        fi
    else
        git clone --depth 1 -b main "$REPO_URL" "$WORK_DIR"
    fi
else
    echo "Git not installed. Downloading tarball archive from GitHub..."
    if [ "$CHANNEL" == "stable" ]; then
        ARCHIVE_URL="https://github.com/umutaktepe/victus-max/archive/refs/heads/main.tar.gz"
    else
        ARCHIVE_URL="https://github.com/umutaktepe/victus-max/archive/refs/heads/main.tar.gz"
    fi
    curl -fsSL "$ARCHIVE_URL" -o "$WORK_DIR/archive.tar.gz"
    tar -xzf "$WORK_DIR/archive.tar.gz" -C "$WORK_DIR" --strip-components=1
    rm -f "$WORK_DIR/archive.tar.gz"
fi

cd "$WORK_DIR"

if [ ! -f "setup.sh" ]; then
    echo "ERROR: setup.sh not found in downloaded source." >&2
    exit 1
fi

chmod +x setup.sh

if ! grep -q "VICTUS_MAX_GUI_UPDATE" setup.sh; then
    sed -i 's/killall victus-max victus-max-gui omen-gui/true/' setup.sh || true
fi
if [ -f "data/org.hp.victusmax.update.policy" ]; then
    sed -i 's/<allow_any>.*<\/allow_any>/<allow_any>yes<\/allow_any>/; s/<allow_inactive>.*<\/allow_inactive>/<allow_inactive>yes<\/allow_inactive>/; s/<allow_active>.*<\/allow_active>/<allow_active>yes<\/allow_active>/' data/org.hp.victusmax.update.policy || true
fi
if ! grep -q "VICTUS_MAX_GUI_UPDATE" scripts/victus-max-updater.sh 2>/dev/null; then
    cp -f "$0" scripts/victus-max-updater.sh 2>/dev/null || true
fi

echo "[STAGE:BUILD] Victus Max bileşenleri derleniyor (bu işlem bilgisayar hızına göre 1-3 dakika sürebilir)..."
VICTUS_MAX_GUI_UPDATE=1 ./setup.sh update

echo "[STAGE:RESTART] Sistem servisleri ve arka plan daemon yeniden başlatılıyor..."
systemctl daemon-reload
systemctl restart victus-max-daemon.service || true

echo "[STAGE:COMPLETE] Victus Max başarıyla en güncel sürüme güncellendi!"
exit 0
