default: integration

integration: check-fmt check clippy test run-example

test:
	cargo test --all ${TEST_ARGS} -- --nocapture

clippy:
	cargo clippy --all --all-targets

check-fmt:
	cargo fmt --all -- --check

check:
	cargo check --all --examples

EXAMPLES := non_threadsafe_demo non_threadsafe_test

run-example:
	for example in ${EXAMPLES} ; do \
		cargo run --example $$example; \
	done
