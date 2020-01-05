default: integration

.PHONY: integration check-fmt check clippy test
integration: check-fmt check clippy test

test:
	cargo test --all --all-features ${TEST_ARGS} -- --nocapture

clippy:
	cargo clippy --all --all-features --all-targets

check-fmt:
	cargo fmt --all -- --check

check:
	cargo check --all --examples
