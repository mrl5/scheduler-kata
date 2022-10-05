SHELL := bash
.ONESHELL:
	.SHELLFLAGS := -eu -o pipefail -c
.DELETE_ON_ERROR:
	MAKEFLAGS += --warn-undefined-variables
	MAKEFLAGS += --no-builtin-rules

install:
	yarn install --frozen-lockfile --ignore-scripts
.PHONY: install

test:
	yarn run test
.PHONY: test
