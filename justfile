default: gates

gates: fmt size last-seen clippy build nvim-test nvim-lint test doc

fmt:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace --all-targets --locked -- -D warnings

build:
    cargo build --workspace --all-targets --locked

test:
    cargo test --workspace --all-targets --no-fail-fast --locked

doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

size:
    @scripts/check-file-sizes-and-test-placement.sh

last-seen:
    @scripts/check-remote-snapshot-readers.sh

test-speed:
    DAM_TESTS_RUN_ALONE_ON_AN_IDLE_MACHINE=1 cargo test --workspace --all-targets --locked -- --test-threads=1

nvim-test:
    #!/usr/bin/env bash
    set -euo pipefail
    throwaway="$(mktemp -d)"
    cd nvim
    HOME="$throwaway" XDG_CONFIG_HOME="$throwaway/config" XDG_DATA_HOME="$throwaway/data" \
      XDG_STATE_HOME="$throwaway/state" XDG_CACHE_HOME="$throwaway/cache" TMPDIR="$throwaway" \
      nvim --headless --clean -l tests/run.lua

nvim-lint:
    cd nvim && stylua --check . && luacheck .
    @! grep -rn 'vim\.system' nvim/lua | grep -v '^nvim/lua/damnit/kernel/process\.lua:' || { echo "vim.system belongs in kernel/process.lua alone" >&2; exit 1; }
