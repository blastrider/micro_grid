use assert_cmd::Command;
use std::fs;

#[test]
fn cli_run_sim_and_output() {
    // requires cargo test to have access to target/debug/mg-cli binary
    let mut cmd = Command::cargo_bin("mg-cli").unwrap();
    cmd.args(&[
        "sim",
        "--name",
        "itest",
        "--seed",
        "7",
        "--n",
        "5",
        "--out",
        "target/itest_ledger.json",
    ]);
    cmd.assert().success();
    let content = fs::read_to_string("target/itest_ledger.json").unwrap();
    assert!(content.contains("buy_order_id") || content.contains("sell_order_id"));
}
