default: integration

integration: check-fmt check clippy test run-example

test:
	cargo test --all --all-features ${TEST_ARGS} -- --nocapture

clippy:
	cargo clippy --all --all-features --all-targets

check-fmt:
	cargo fmt --all -- --check

check:
	cargo check --all --examples

run-example:
	cargo run --example non_threadsafe_demo
