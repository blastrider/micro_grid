#![forbid(unsafe_code)]
use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use mg_lib::{Order, OrderBook, Side, io, ledger::Ledger, matcher::Matcher};
use rust_decimal::Decimal;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

/// mg-cli: minimal orchestrator for local matching
#[derive(Parser)]
#[command(name = "mg-cli")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Load orders (csv/json) and run matching
    Run {
        /// input file (csv or json)
        #[arg(short, long)]
        input: PathBuf,

        /// output ledger file (json)
        #[arg(short, long)]
        out: Option<PathBuf>,

        /// run id
        #[arg(long)]
        run_id: Option<String>,
    },
    /// Generate a simulation scenario and run it
    Sim {
        #[arg(long, default_value = "demo")]
        name: String,
        #[arg(long, default_value_t = 42)]
        seed: u64,
        #[arg(long, default_value_t = 20)]
        n: usize,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Create a single order (prints JSON; optionally write to file)
    CreateOrder {
        /// tenant id
        #[arg(long)]
        tenant: String,

        /// side: buy or sell
        #[arg(long)]
        side: String,

        /// kWh (decimal with up to 4 decimals), e.g. 2.5000
        #[arg(long)]
        kwh: String,

        /// price (decimal), e.g. 0.1500
        #[arg(long)]
        price: String,

        /// optional order id (if omitted a generated id is used)
        #[arg(long)]
        id: Option<String>,

        /// write order JSON to this path (optional)
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    // initialize tracing if feature enabled (tracing-subscriber is optional)
    let _ = std::panic::catch_unwind(|| {
        let _ = tracing_subscriber::fmt::try_init();
    });

    match cli.cmd {
        Commands::Run { input, out, run_id } => {
            let orders = io::load_orders(&input).context("loading orders")?;
            let mut book = OrderBook::from(orders);
            let matches = Matcher::match_book(&mut book, run_id);
            let mut ledger = Ledger::new();
            ledger.extend(matches);
            // branch Run
            if let Some(p) = out {
                ledger.to_file(&p).context("writing ledger")?;
                println!("ledger written to {}", p.display());
            } else {
                println!(
                    "{}",
                    ledger
                        .to_json()
                        .unwrap_or_else(|_| "failed serialize".into())
                );
            }
        }
        Commands::Sim { name, seed, n, out } => {
            let orders = mg_lib::simulate::generate_scenario(&name, seed, n);
            let mut book = OrderBook::from(orders);
            let matches = Matcher::match_book(&mut book, Some(format!("sim-{}-{}", name, seed)));
            let mut ledger = Ledger::new();
            ledger.extend(matches);
            if let Some(p) = out {
                ledger.to_file(&p).context("writing ledger")?;
                println!("ledger written to {}", p.display());
            } else {
                println!(
                    "{}",
                    ledger
                        .to_json()
                        .unwrap_or_else(|_| "serialize failed".into())
                );
            }
        }
        Commands::CreateOrder {
            tenant,
            side,
            kwh,
            price,
            id,
            out,
        } => {
            // parse side
            let side_enum = match side.to_lowercase().as_str() {
                "buy" | "b" => Side::Buy,
                "sell" | "s" => Side::Sell,
                other => anyhow::bail!("invalid side '{}', expected buy|sell", other),
            };

            // parse decimals
            let kwh_dec = Decimal::from_str(&kwh).context("parsing kwh as decimal")?;
            let price_dec = Decimal::from_str(&price).context("parsing price as decimal")?;

            // id default
            let order_id = id.unwrap_or_else(|| {
                // timestamp en millisecondes + petit sel aléatoire pour unicité
                format!(
                    "order-{}-{}",
                    chrono::Utc::now().timestamp_millis(),
                    rand::random::<u32>()
                )
            });

            let order = Order {
                id: order_id,
                tenant_id: tenant,
                side: side_enum,
                kwh: kwh_dec,
                price: price_dec,
                timestamp: Utc::now(),
                remaining_kwh: kwh_dec,
            };

            // print pretty JSON to stdout
            let json = serde_json::to_string_pretty(&order).context("serialize order to json")?;
            println!("{}", json);

            // if out given write to file (overwrite)
            if let Some(p) = out {
                let mut f =
                    File::create(&p).with_context(|| format!("creating file {}", p.display()))?;
                f.write_all(json.as_bytes())
                    .context("writing order json to file")?;
                println!("order written to {}", p.display());
            }
        }
    }

    Ok(())
}
