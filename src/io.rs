// src/io.rs
use crate::matcher::{Order, Side};
use chrono::{DateTime, Utc};
use csv::StringRecord;
use rust_decimal::Decimal;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("csv error: {0}")]
    Csv(#[from] csv::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("parse error: {0}")]
    Parse(String),
}

impl From<rust_decimal::Error> for LoadError {
    fn from(e: rust_decimal::Error) -> Self {
        LoadError::Parse(e.to_string())
    }
}

impl From<chrono::ParseError> for LoadError {
    fn from(e: chrono::ParseError) -> Self {
        LoadError::Parse(e.to_string())
    }
}

/// Simple CSV header: id,tenant_id,side,kwh,price,timestamp (RFC3339)
pub fn load_orders<P: AsRef<Path>>(path: P) -> Result<Vec<Order>, LoadError> {
    let p = path.as_ref();
    let ext = p
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    let content = fs::read_to_string(p)?;

    match ext.as_str() {
        "json" => {
            let mut orders: Vec<Order> = serde_json::from_str(&content)?;
            for o in &mut orders {
                if o.remaining_kwh == Decimal::ZERO {
                    o.remaining_kwh = o.kwh;
                }
                o.normalize();
                o.validate().map_err(LoadError::Parse)?;
            }
            Ok(orders)
        }
        "csv" => {
            let mut rdr = csv::Reader::from_reader(content.as_bytes());
            let mut res = Vec::new();
            for result in rdr.records() {
                let record = result?;
                let o = parse_csv_record(&record)?;
                res.push(o);
            }
            Ok(res)
        }
        _ => Err(LoadError::Parse(format!("unsupported extension: {}", ext))),
    }
}

fn parse_csv_record(rec: &StringRecord) -> Result<Order, LoadError> {
    // tolerate with indexes
    let id = rec
        .get(0)
        .ok_or_else(|| LoadError::Parse("missing id".into()))?
        .to_string();
    let tenant_id = rec
        .get(1)
        .ok_or_else(|| LoadError::Parse("missing tenant_id".into()))?
        .to_string();
    let side_s = rec
        .get(2)
        .ok_or_else(|| LoadError::Parse("missing side".into()))?;
    let side = match side_s.to_lowercase().as_str() {
        "buy" | "b" => Side::Buy,
        "sell" | "s" => Side::Sell,
        other => return Err(LoadError::Parse(format!("invalid side '{}'", other))),
    };
    let kwh = rec
        .get(3)
        .ok_or_else(|| LoadError::Parse("missing kwh".into()))?;
    let kwh = Decimal::from_str_exact(kwh)?;
    let price = rec
        .get(4)
        .ok_or_else(|| LoadError::Parse("missing price".into()))?;
    let price = Decimal::from_str_exact(price)?;
    let ts = rec
        .get(5)
        .ok_or_else(|| LoadError::Parse("missing timestamp".into()))?;
    let timestamp = DateTime::parse_from_rfc3339(ts)?.with_timezone(&Utc);

    let mut o = Order {
        id,
        tenant_id,
        side,
        kwh,
        price,
        timestamp,
        remaining_kwh: kwh,
    };
    o.normalize();
    o.validate().map_err(LoadError::Parse)?;
    Ok(o)
}
