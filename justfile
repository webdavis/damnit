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
    @scripts/check-file-sizes-and-test-placement.sh
