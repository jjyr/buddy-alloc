.PHONY: default integration test fuzz

default: integration

integration: check-fmt check clippy test test-release run-example

test:
	cargo test --all ${TEST_ARGS} -- --nocapture

test-release:
	cargo test --release --all ${TEST_ARGS} -- --nocapture

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

fuzz:
	cargo +nightly fuzz run chaos

