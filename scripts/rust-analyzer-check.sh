#!/usr/bin/env bash
# Fails if rust-analyzer reports any diagnostic for the workspace.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
out="$(rust-analyzer diagnostics . 2>/dev/null || true)"
if printf '%s\n' "$out" | grep -qE '^(Error|Warning|WeakWarning)'; then
    printf '%s\n' "$out"
    exit 1
fi
