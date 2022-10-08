SHELL := bash
.ONESHELL:
	.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
	MAKEFLAGS += --warn-undefined-variables
	MAKEFLAGS += --no-builtin-rules

install: build
	cargo install --path crates/server/
.PHONY: install

build:
	cargo build --release
.PHONY: build

serve:
	./target/release/server
.PHONY: build

test:
	cargo test
.PHONY: test

setup-e2e-env:
	cd scripts
	make install
.PHONY: setup-e2e-env

e2e: setup-e2e-env
	cd scripts
	make test
.PHONY: e2e

lint: fmt
	cargo clippy --fix --allow-staged
.PHONY: lint

fmt:
	rustfmt crates/**/src/*.rs
	#rustfmt crates/**/src/**/*.rs
.PHONY: fmt
