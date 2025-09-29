use mg_lib::{Ledger, Matcher, OrderBook, simulate};

fn main() {
    let orders = simulate::generate_scenario("example", 123, 10);
    let mut book = OrderBook::from(orders);
    let matches = Matcher::match_book(&mut book, Some("example-run".into()));
    let mut ledger = Ledger::new();
    ledger.extend(matches);
    println!("{}", ledger.to_json().unwrap());
}
