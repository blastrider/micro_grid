.PHONY: fmt run lint test run-cli run-sim snapshot create-order

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

snapshot:
	mkdir -p ./scripts/tmp
	bash ./scripts/snapshot.sh ./scripts/tmp

# Create an order quickly:
# Usage examples:
#   make create-order TENANT=T1 SIDE=sell KWH=2.5000 PRICE=0.1500 OUT=examples/order.json
#   make create-order             # uses defaults
TENANT ?= T1
SIDE   ?= buy
KWH    ?= 1.0000
PRICE  ?= 0.1000
ID     ?=
OUT    ?=

create-order:
	cargo run --features cli --bin mg-cli -- create-order \
		--tenant "$(TENANT)" \
		--side "$(SIDE)" \
		--kwh "$(KWH)" \
		--price "$(PRICE)" \
		$(if $(ID),--id "$(ID)") \
		$(if $(OUT),--out "$(OUT)")
