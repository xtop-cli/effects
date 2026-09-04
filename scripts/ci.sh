#!/usr/bin/env bash
# Local CI for the xtop effects repo (Cargo workspace since M5).
#
# Intentionally NOT wired into git: no GitHub Actions, no git hooks. Run it
# yourself from the repo root:
#
#   ./scripts/ci.sh            # run every stage
#   ./scripts/ci.sh fmt        # run one stage
#
# Stages: fmt | clippy | check | test
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ ! -f Cargo.toml ]]; then
    echo "[ci] no Cargo.toml at repo root; nothing to run."
    exit 0
fi

stages=(fmt clippy check test)
requested=("$@")
if [[ ${#requested[@]} -eq 0 ]]; then
    requested=("${stages[@]}")
fi

fmt() {
    echo "==> fmt (format check)"
    cargo fmt --all -- --check
}

clippy() {
    echo "==> clippy (workspace, all targets, warnings denied)"
    cargo clippy --workspace --all-targets -- -D warnings
}

check() {
    echo "==> check (workspace)"
    cargo check --workspace
}

test() {
    echo "==> test (workspace)"
    cargo test --workspace
}

for stage in "${requested[@]}"; do
    case "$stage" in
        fmt) fmt ;;
        clippy) clippy ;;
        check) check ;;
        test) test ;;
        *)
            echo "[ci] unknown stage: $stage (expected one of: ${stages[*]})" >&2
            exit 1
            ;;
    esac
done

echo "[ci] all stages passed."
