cargo fmt --all
cargo build
cargo +1.88.0 clippy --all-targets --all-features -- -D warnings

# cargo build --target x86_64-pc-windows-gnu --release