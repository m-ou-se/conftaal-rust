use std::env;
use std::path::PathBuf;
use std::process::{Command, exit};

fn cargo_dir() -> PathBuf {
	let mut dir = env::current_exe().unwrap();
	dir.pop(); dir.pop(); dir
}

#[test]
fn main() {
	let exe = cargo_dir().join(format!("conftaal-parse{}", env::consts::EXE_SUFFIX));
	if !Command::new("tests/test.sh").arg(exe).status().unwrap().success() {
	println!("asdf");
		exit(1);
	}
}
