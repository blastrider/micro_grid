# migro_grid — Micro-grid local exchange (MVP)

Bibliothèque Rust + CLI légère pour simuler un marché local d’échange d’énergie (orderbook, matching, ledger).
But : fournir une **lib réutilisable** (algorithme : orderbook, matching, clearing) et un **binaire CLI** d’orchestration local pour tester des règles économiques.

---

## Éléments clés

* Crate unique exposant `lib` (API publique) et binaire `mg-cli` dans `src/bin/`.
* `#![forbid(unsafe_code)]` — pas d'`unsafe`.
* Précision kWh : **4 décimales** (0.0001 kWh).
* Règle de matching : **price-time priority** — buy : prix décroissant ; sell : prix croissant ; tie-breaker : timestamp FIFO.
* Prix de transaction : **prix de l’ordre passif (resting order)**.
* Tests unitaires et d’intégration inclus.

---

## Prérequis

* Rust stable toolchain (ex. `rustup default stable`), `cargo` disponible.
* `make` (optionnel, si tu veux utiliser le `Makefile` fourni).

---

## Build & exécution (rapide)

> Note : les dépendances CLI (`clap`, `anyhow`, `tracing-subscriber`) sont placées sous la feature `cli`. Active-la pour compiler / exécuter le binaire.

```bash
# build complet (lib + cli)
cargo build --features cli

# exécuter l'aide du CLI
cargo run --features cli --bin mg-cli -- --help

# exécuter une simulation simple (génère ledger.json)
cargo run --features cli --bin mg-cli -- sim --name demo --seed 1 --n 20 --out target/ledger.json

# charger un fichier orders.csv et matcher -> ledger.json
cargo run --features cli --bin mg-cli -- run --input examples/orders.csv --out target/ledger.json
```

Si tu préfères, utilise le Makefile :

```bash
make fmt
make lint      # exécute clippy avec --features cli
make run-sim   # target configuré pour --features cli
make test      # exécute tests avec --features cli
```

---

## Format d’entrée (CSV)

CSV attendu (entête optionnelle, sinon colonnes indexées) :

```
id,tenant_id,side,kwh,price,timestamp
o1,T1,buy,2.5000,0.1500,2025-09-29T12:00:00Z
o2,T2,sell,1.0000,0.1400,2025-09-29T12:01:00Z
```

* `side` : `buy` | `sell` (ou `b` / `s`).
* `kwh` et `price` : décimales au format `0.0000`.
* `timestamp` : RFC3339 (UTC recommandé).

---

## API publique (usage minimal)

La logique métier vit dans la lib (pub API stable) :

```rust
use mg_lib::{io, OrderBook, Matcher, Ledger};

fn main() -> anyhow::Result<()> {
    // charge commandes depuis CSV/JSON
    let orders = mg_lib::io::load_orders("examples/orders.csv")?;
    let mut book = OrderBook::from(orders);
    // lance le matching
    let matches = mg_lib::matcher::Matcher::match_book(&mut book, Some("run-1".into()));
    // crée ledger
    let mut ledger = Ledger::new();
    ledger.extend(matches);
    // écrit JSON
    ledger.to_file("target/ledger.json")?;
    Ok(())
}
```

API exposée (fichiers principaux) :

* `mg_lib::Order` — structure ordres (id, tenant_id, side, kwh, price, timestamp, remaining_kwh).
* `mg_lib::OrderBook` — construction depuis `Vec<Order>`.
* `mg_lib::matcher::Matcher::match_book(&mut OrderBook, Option<String>) -> Vec<MatchRecord>`.
* `mg_lib::Ledger` — append-only, `to_json()`, `to_file()`.

---

## Exemples

* `examples/basic.rs` — démonstration d’un run simulé.
* `examples/orders.csv` — jeu d’exemple.

---

## Tests

La suite de tests couvre matcher, ledger et tests d’intégration CLI.

```bash
# exécuter les tests (active la feature cli pour que le binaire compile dans les tests d'intégration)
cargo test --features cli
```

---

## Choix de conception importants

* **Précision** : arrondissements systématiques sur 4 décimales (kWh).
* **Idempotence** : `MatchRecord` contient un `run_id` (optionnel) pour tracer/éventuellement réappliquer de façon idempotente.
* **Prix** : trade price = passive order price (documenté).
* **Robustesse** : validation stricte en `io::load_orders()` ; erreurs publiques via `thiserror`.
* **Observabilité** : la lib accepte `tracing` (et le binaire initialise `tracing_subscriber` si la feature `cli` est activée).

---

## Roadmap (post-MVP)

* frais/fees, prorata, settlement batch.
* persistance d’idempotence (store run_id).
* Web bindings / wasm / API REST.
* publication crates.io (separation lib vs bin packaging si nécessaire).
* property tests / fuzzing pour matcher.

---

## Troubleshooting rapide

* `cannot find attribute 'command'` / `unresolved import 'clap'` → compile/execute avec `--features cli` ou rendre `clap` non-optionnel dans `Cargo.toml`.
* `Makefile: *** séparateur manquant` → lignes de recette **doivent** commencer par une vraie tabulation (pas d’espaces).
* CRLF/Windows → `dos2unix Makefile` / `sed -i 's/\r$//' Makefile`.
* warnings clippy stricts : `cargo clippy --features cli -- -D warnings`. Ajuste code si clippy réclame.

---

## Contribuer

1. Fork → branch feature → PR.
2. Respecter : `cargo fmt`, `cargo clippy -- -D warnings`, tests `cargo test --features cli`.
3. Tenir la public API stable (semver) : modifications breaking → major bump.

---

## Licence

MIT OR Apache-2.0 — vérifier `Cargo.toml` pour la version exacte.

---

## Contact

Pour bugs / demandes d’amélioration : ouvre une issue dans le repo GitHub (champ `repository` dans `Cargo.toml`).