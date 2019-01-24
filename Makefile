all: test
.PHONY: all

build:
	@cargo build --all-features
.PHONY: build

doc:
	@cargo doc
.PHONY: doc

test: cargotest
.PHONY: test

cargotest:
	@cargo test --all-features
.PHONY: cargotest

format-check:
	@rustup component add rustfmt 2> /dev/null
	@cargo fmt -- --check
.PHONY: format-check

format:
	@rustup component add rustfmt 2> /dev/null
	@cargo fmt
.PHONY: format

lint:
	@rustup component add clippy 2> /dev/null
	@cargo clippy --all-features --tests -- -D clippy::all
.PHONY: lint
