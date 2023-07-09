TMRW := `date -u -Iseconds -d"+1days"`

dev-tools:
    cargo install hurl

local-api:
    cargo run

test: test-unit test-api

test-unit:
    cargo test

test-api:
    hurl --test --variable tomorrow={{TMRW}} ./tests/*.hurl

lint: fmt
    cargo clippy --fix --allow-staged

fmt:
    rustfmt crates/**/src/*.rs
