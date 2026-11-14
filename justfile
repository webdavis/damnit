default: gates

gates: fmt clippy build test doc size

fmt:
    cargo fmt --all -- --check

clippy:
    cargo clippy --workspace --all-targets --locked -- -D warnings

build:
    cargo build --locked

test:
    cargo test --workspace --no-fail-fast --locked

doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

size:
    #!/usr/bin/env bash
    set -euo pipefail
    allowed_over_150=(
      crates/dam-cli/src/error/refusal_text.rs # one exhaustive match over every refusal
      crates/dam-adapters/src/sqlite/codec/notices.rs # one two-way codec over one row enum
    )
    is_test_file() { [[ "$1" == */tests.rs || "$1" == *_tests.rs || "$1" == */tests/* ]]; }
    is_allowed_over_150() { [[ " ${allowed_over_150[*]} " == *" $1 "* ]]; }
    fail=0
    while IFS= read -r f; do
      total=$(($(wc -l < "$f")))
      if [ "$total" -gt 250 ]; then echo "OVER 250: $f ($total)"; fail=1; fi
      if [[ "$f" != crates/*/src/* ]] && [[ "$f" != crates/*/tests/* ]]; then continue; fi
      if is_test_file "$f"; then
        if [ "$total" -gt 200 ]; then echo "TEST OVER 200: $f ($total)"; fail=1; fi
        continue
      fi
      if [[ "$f" != crates/*/src/* ]]; then continue; fi
      if [ "$total" -gt 150 ] && ! is_allowed_over_150 "$f"; then echo "OVER 150: $f ($total)"; fail=1; fi
      if awk '/^[[:space:]]*#\[cfg\(test\)\]/ { armed = 1; next } armed && /^[[:space:]]*(pub(\([^)]*\))? )?mod [a-z_0-9]+[[:space:]]*\{/ { found = 1 } NF { armed = 0 } END { exit !found }' "$f"; then
        echo "INLINE TEST MODULE: $f"; fail=1
      fi
    done < <(git ls-files '*.rs')
    exit "$fail"
