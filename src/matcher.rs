// src/matcher.rs
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Precision: 4 decimal places for kWh
const KWH_PRECISION: u32 = 4;

fn max_kwh() -> Decimal {
    Decimal::new(1_000_000_i64, 0) // 1_000_000 kWh upper bound
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub tenant_id: String,
    pub side: Side,
    pub kwh: Decimal,
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
    #[serde(default)]
    pub remaining_kwh: Decimal,
}

impl Order {
    pub fn normalize(&mut self) {
        self.remaining_kwh = self.remaining_kwh.round_dp(KWH_PRECISION);
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.kwh <= Decimal::ZERO {
            return Err("kwh must be > 0".into());
        }
        if self.kwh > max_kwh() {
            return Err("kwh exceeds max allowed".into());
        }
        if self.price <= Decimal::ZERO {
            return Err("price must be > 0".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRecord {
    pub buy_order_id: String,
    pub sell_order_id: String,
    pub kwh: Decimal,
    pub price: Decimal,
    pub timestamp: DateTime<Utc>,
    pub run_id: Option<String>,
}

pub struct OrderBook {
    pub buys: Vec<Order>,
    pub sells: Vec<Order>,
}

impl From<Vec<Order>> for OrderBook {
    fn from(mut v: Vec<Order>) -> Self {
        for o in &mut v {
            if o.remaining_kwh == Decimal::ZERO {
                o.remaining_kwh = o.kwh;
            }
            o.remaining_kwh = o.remaining_kwh.round_dp(KWH_PRECISION);
        }

        // partition into buys and sells without eager cloning
        let (buys, sells): (Vec<Order>, Vec<Order>) =
            v.into_iter().partition(|o| matches!(o.side, Side::Buy));

        // sort buys: price desc, timestamp asc
        let mut buys = buys;
        buys.sort_by(|a, b| match b.price.cmp(&a.price) {
            Ordering::Equal => a.timestamp.cmp(&b.timestamp),
            other => other,
        });

        // sort sells: price asc, timestamp asc
        let mut sells = sells;
        sells.sort_by(|a, b| match a.price.cmp(&b.price) {
            Ordering::Equal => a.timestamp.cmp(&b.timestamp),
            other => other,
        });

        OrderBook { buys, sells }
    }
}

pub struct Matcher {}

impl Matcher {
    /// Match the orderbook and return executed matches.
    /// trade_price chosen as the passive order price (the resting order).
    pub fn match_book(book: &mut OrderBook, run_id: Option<String>) -> Vec<MatchRecord> {
        let mut matches = Vec::new();

        while let (Some(buy), Some(sell)) =
            (book.buys.first().cloned(), book.sells.first().cloned())
        {
            // stop if best buy price < best sell price
            if buy.price < sell.price {
                break;
            }

            // determine trade quantity
            let trade_qty = std::cmp::min(buy.remaining_kwh, sell.remaining_kwh);
            let trade_qty = trade_qty.round_dp(KWH_PRECISION);
            if trade_qty <= Decimal::ZERO {
                break;
            }

            let trade_price = sell.price;

            let record = MatchRecord {
                buy_order_id: buy.id.clone(),
                sell_order_id: sell.id.clone(),
                kwh: trade_qty,
                price: trade_price,
                timestamp: Utc::now(),
                run_id: run_id.clone(),
            };

            matches.push(record);

            // update orders (mutate first elements)
            {
                let b0 = &mut book.buys[0];
                b0.remaining_kwh = (b0.remaining_kwh - trade_qty).round_dp(KWH_PRECISION);
            }
            {
                let s0 = &mut book.sells[0];
                s0.remaining_kwh = (s0.remaining_kwh - trade_qty).round_dp(KWH_PRECISION);
            }

            // remove filled orders
            if book.buys[0].remaining_kwh <= Decimal::ZERO {
                book.buys.remove(0);
            }
            if !book.sells.is_empty() && book.sells[0].remaining_kwh <= Decimal::ZERO {
                book.sells.remove(0);
            }
        }

        matches
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use rust_decimal::prelude::FromStr;

    #[test]
    fn test_simple_fill() {
        let buy = Order {
            id: "b1".into(),
            tenant_id: "t".into(),
            side: Side::Buy,
            kwh: Decimal::from_str("5").unwrap(),
            price: Decimal::from_str("0.20").unwrap(),
            timestamp: Utc::now(),
            remaining_kwh: Decimal::ZERO,
        };
        let sell = Order {
            id: "s1".into(),
            tenant_id: "t".into(),
            side: Side::Sell,
            kwh: Decimal::from_str("3").unwrap(),
            price: Decimal::from_str("0.18").unwrap(),
            timestamp: Utc::now(),
            remaining_kwh: Decimal::ZERO,
        };

        let mut book = OrderBook::from(vec![buy, sell]);
        let matches = Matcher::match_book(&mut book, Some("run1".into()));
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].kwh, Decimal::from_str("3").unwrap());
        assert_eq!(book.buys.len(), 1);
        assert_eq!(book.buys[0].remaining_kwh, Decimal::from_str("2").unwrap());
    }

    #[test]
    fn test_no_cross() {
        let buy = Order {
            id: "b1".into(),
            tenant_id: "t".into(),
            side: Side::Buy,
            kwh: Decimal::from_str("1").unwrap(),
            price: Decimal::from_str("0.10").unwrap(),
            timestamp: Utc::now(),
            remaining_kwh: Decimal::ZERO,
        };
        let sell = Order {
            id: "s1".into(),
            tenant_id: "t".into(),
            side: Side::Sell,
            kwh: Decimal::from_str("1").unwrap(),
            price: Decimal::from_str("0.20").unwrap(),
            timestamp: Utc::now(),
            remaining_kwh: Decimal::ZERO,
        };

        let mut book = OrderBook::from(vec![buy, sell]);
        let matches = Matcher::match_book(&mut book, None);
        assert!(matches.is_empty());
    }
}
