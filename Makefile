.PHONY: check

build:
    CC=gcc cargo build

test:
	CC=gcc cargo test

lint:
	CC=gcc cargo clippy -- -D warnings

check: test lint
	cargo fmt --check
	CC=gcc cargo clippy -- -D warnings

.DEFAULT_GOAL := check
