#!/usr/bin/env bash
# Auto-push to GitHub on file changes
REPO="/home/ghostshell/Desktop/NganjoOS-v1.0-Lite-3/nganjo-os"
cd "$REPO"

echo "Watching for changes in $REPO ..."

while true; do
    # Check if there's anything to commit
    if ! git diff --quiet || ! git diff --cached --quiet || [ -n "$(git ls-files --others --exclude-standard)" ]; then
        echo "[$(date '+%H:%M:%S')] Changes detected — committing and pushing..."
        git add .
        git commit -m "auto: $(date '+%Y-%m-%d %H:%M:%S')"
        git push origin main
        echo "[$(date '+%H:%M:%S')] Pushed."
    fi
    sleep 10
done
