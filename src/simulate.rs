// src/simulate.rs
use crate::matcher::{Order, Side};
use chrono::Utc;
use rand::{Rng, SeedableRng, rngs::StdRng};
use rust_decimal::Decimal;

pub fn generate_scenario(name: &str, seed: u64, n: usize) -> Vec<Order> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let side = if rng.gen_bool(0.5) {
            Side::Buy
        } else {
            Side::Sell
        };
        // kWh between 0.0001 and 1000000 (scaled by 10^4)
        let k_int: i64 = rng.gen_range(1..=1_000_000); // corresponds to 0.0001 .. 100.0000 if you prefer smaller range, adjust
        let kwh = Decimal::new(k_int, 4);

        // price between 0.05 and 0.50 -> scaled by 10^4
        let p_int: i64 = rng.gen_range(500..=5000); // 0.0500 .. 0.5000
        let price = Decimal::new(p_int, 4);

        let o = Order {
            id: format!("{}-{}-{}", name, seed, i),
            tenant_id: format!("t{}", rng.gen_range(1..5)),
            side,
            kwh,
            price,
            timestamp: Utc::now(),
            remaining_kwh: kwh,
        };
        out.push(o);
    }
    out
}
