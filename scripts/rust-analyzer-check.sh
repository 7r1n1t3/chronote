#!/usr/bin/env bash
# Fails if rust-analyzer reports any diagnostic for this workspace's own files.
# `inactive_code` is ignored: it only flags #[cfg(test)] blocks.
set -euo pipefail
root="$(git rev-parse --show-toplevel)"
cd "$root"
out="$(rust-analyzer diagnostics . 2>/dev/null || true)"
hits="$(printf '%s\n' "$out" | grep -F "file $root/" | grep -E ': (Error|Warning|WeakWarning) ' | grep -v '"inactive_code"' || true)"
if [[ -n "$hits" ]]; then
    printf '%s\n' "$hits"
    exit 1
fi
