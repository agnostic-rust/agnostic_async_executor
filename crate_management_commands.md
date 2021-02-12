# Run tests
You need to comment out the wasm dev-dependencies and uncomment dev-dependencies, then run:
cargo test

# Run wasm tests
You need to comment out the dev-dependencies and uncomment wasm dev-dependencies, then run:
wasm-pack test --firefox --headless

# Build the docs
cargo doc --features "test, async_std_executor_with_time, tokio_executor_with_time, smol_executor_with_time, futures_executor_with_time, wasm_bindgen_executor_with_time"

# Publish the to crates.io
Delete de target directory, then run:
cargo publish --features "test, async_std_executor_with_time, tokio_executor_with_time, smol_executor_with_time, futures_executor_with_time, wasm_bindgen_executor_with_time"