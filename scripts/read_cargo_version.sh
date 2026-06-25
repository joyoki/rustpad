#!/bin/bash
# Print the root package version from Cargo.toml (via cargo pkgid).
set -euo pipefail

PROJECT_DIR="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

cd "$PROJECT_DIR"
version="$(cargo pkgid 2>/dev/null | sed 's/.*#//')"
if [ -z "$version" ]; then
    echo "error: could not read package version from Cargo.toml" >&2
    exit 1
fi
printf '%s' "$version"
