// src/ledger.rs
use crate::matcher::MatchRecord;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ledger {
    pub entries: Vec<MatchRecord>,
}

impl Ledger {
    pub fn new() -> Self {
        Ledger {
            entries: Vec::new(),
        }
    }

    pub fn append(&mut self, r: MatchRecord) {
        self.entries.push(r);
    }

    pub fn extend(&mut self, rows: Vec<MatchRecord>) {
        self.entries.extend(rows);
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self.entries)
    }

    pub fn to_file<P: AsRef<Path>>(&self, p: P) -> Result<(), io::Error> {
        let json = self
            .to_json()
            .map_err(|e| io::Error::other(e.to_string()))?;
        let mut f = File::create(p)?;
        f.write_all(json.as_bytes())?;
        Ok(())
    }
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matcher::MatchRecord;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use std::str::FromStr;

    #[test]
    fn ledger_roundtrip() {
        let mut l = Ledger::new();
        let m = MatchRecord {
            buy_order_id: "b".into(),
            sell_order_id: "s".into(),
            kwh: Decimal::from_str("1.2345").unwrap(),
            price: Decimal::from_str("0.12").unwrap(),
            timestamp: Utc::now(),
            run_id: None,
        };
        l.append(m);
        let json = l.to_json().unwrap();
        assert!(json.contains("buy_order_id"));
    }
}
