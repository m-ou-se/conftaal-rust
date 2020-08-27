use std::process::{exit, Command};

#[test]
fn main() {
    if !Command::new("tests/test.sh")
        .arg(env!("CARGO_BIN_EXE_conftaal-parse"))
        .status()
        .unwrap()
        .success()
    {
        exit(1);
    }
}
