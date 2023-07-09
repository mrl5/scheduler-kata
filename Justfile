dev-tools:
    cargo install hurl

local-api:
    cargo run

test: test-unit test-api

test-unit:
    cargo test

test-api:
    hurl --test ./tests/*.hurl

lint: fmt
    cargo clippy --fix --allow-staged

fmt:
    rustfmt crates/**/src/*.rs
