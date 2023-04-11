.DEFAULT_GOAL := build

.PHONY: build
build: target/doc
	cargo build

.PHONY: release
release: target/doc
	cargo build --release

.PHONY: test
test:
	cargo test

target/doc: Cargo.*
	cargo doc

.PHONY: doc
doc:
	$(MAKE) -B target/doc

.PHONY: upgrade
upgrade:
	cargo upgrade --incompatible

.PHONY: lint
lint:
	cargo fmt -- --check
	cargo +nightly clippy -- -Wclippy::pedantic

.PHONY: fix
fix:
	cargo +nightly clippy --fix --allow-staged -- -Wclippy::pedantic
	cargo fmt

.PHONY: build-test
build-test:
	cargo test --no-run
