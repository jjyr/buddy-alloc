default: integration

.PHONY: integration check-fmt clippy test
integration: check-fmt clippy test

test:
	cargo test --all --all-features ${TEST_ARGS} -- --nocapture

clippy:
	cargo clippy --all --all-features --all-targets

check-fmt:
	cargo fmt --all -- --check
