#!/usr/bin/env bash
# Fails if rust-analyzer reports any diagnostic for this workspace's own files.
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
cd "$root"
out="$(rust-analyzer diagnostics . 2>/dev/null || true)"
hits="$(printf '%s\n' "$out" | grep -F "file $root/" | grep -E ': (Error|Warning|WeakWarning) ' || true)"
if [[ -n "$hits" ]]; then
    printf '%s\n' "$hits"
    exit 1
fi
