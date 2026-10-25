#!/usr/bin/env bash
# Genera latest.json a partir de una release ya publicada en GitHub.
# Uso: latest-json.sh <tag> [salida]
set -euo pipefail

TAG="${1:?uso: latest-json.sh <tag> [salida]}"
OUTPUT="${2:-latest.json}"
REPO="${REPO:-ortisec/TChat}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT

gh release view "$TAG" --repo "$REPO" --json tagName,publishedAt,assets > "$workdir/release.json"
gh release download "$TAG" --repo "$REPO" --pattern '*.sha256' --dir "$workdir" >/dev/null 2>&1 || true

python3 "$SCRIPT_DIR/latest_json.py" "$workdir" "$OUTPUT"
