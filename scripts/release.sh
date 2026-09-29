#!/usr/bin/env bash
# ==============================================================================
# Victus Max Release Helper
# Synchronizes Cargo versions, tags the release, and prepares git push.
# ==============================================================================
set -euo pipefail

NEW_VER="${1:-}"

if [ -z "$NEW_VER" ]; then
    CURRENT_VER=$(sed -n 's/^version = "\(.*\)"/\1/p' src/victus-max-gui/Cargo.toml | head -1)
    echo "Usage: ./scripts/release.sh <new-version> (e.g. 2.1.4)"
    echo "Current version: $CURRENT_VER"
    exit 1
fi

CLEAN_VER="${NEW_VER#v}"
TAG="v${CLEAN_VER}"

echo "Preparing release $TAG..."

# Update versions in all Cargo.toml
for file in src/*/Cargo.toml; do
    sed -i "s/^version = \".*\"/version = \"$CLEAN_VER\"/" "$file"
    echo "Updated $file to $CLEAN_VER"
done

# Update internal omen-types version requirements
sed -i -E 's/(omen-types = \{ )version = "[^"]*", (path = "\.\.\/victus-max-types")/\1version = "'"$CLEAN_VER"'", \2/' src/*/Cargo.toml
echo "Updated omen-types dependency versions to $CLEAN_VER"

cargo check --workspace

git add src/*/Cargo.toml Cargo.lock 2>/dev/null || git add src/*/Cargo.toml
git commit -m "chore(release): bump version to $TAG"
git tag -a "$TAG" -m "Release $TAG"

echo ""
echo "✅ Release $TAG committed and tagged locally."
echo "To publish, run:"
echo "    git push origin main --tags"
