#![forbid(unsafe_code)]
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use mg_lib::{OrderBook, io, ledger::Ledger, matcher::Matcher};
use std::path::PathBuf;

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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt::init();

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
    }

    Ok(())
}
