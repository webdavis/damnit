# justfile
default: gates

gates: fmt clippy test size

fmt:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo test --workspace --no-fail-fast

# Physical lines after rustfmt; the count the clean-code Rust standard names.
size:
    #!/usr/bin/env bash
    set -euo pipefail
    fail=0
    while IFS= read -r f; do
      total=$(wc -l < "$f")
      if [ "$total" -gt 500 ]; then echo "OVER 500: $f ($total)"; fail=1; fi
    done < <(git ls-files 'crates/*/src/*.rs' 'crates/*/src/**/*.rs' 'crates/*/tests/*.rs')
    exit "$fail"
