default: gates

gates: fmt size last-seen clippy build test doc

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
