.PHONY: fmt run lint test run-cli run-sim

fmt:
	cargo fmt

run:
	clear && cargo fmt && cargo run --features cli

lint:
	clear
	cargo fmt -- --check
	cargo clippy --features cli -- -D warnings

test:
	clear
	cargo test --features cli --locked

run-cli:
	cargo run --features cli --bin mg-cli -- run --input examples/orders.csv --out target/ledger.json

run-sim:
	cargo run --features cli --bin mg-cli -- sim --name demo --seed 1 --n 20 --out target/ledger.json
