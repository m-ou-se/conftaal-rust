extern crate conftaal;

use conftaal::parse::Parser;
use conftaal::parse::matcher::{Matcher, MatchMode};

fn main() {
	let mut p = Parser{
		source: "a * (a.bla + b)"
	};

	println!("Parsing: {}", p.source);

	match p.parse_expression(&Matcher::new(MatchMode::EndOfFile)) {
		Ok(expr) => println!("Result: {:?}", expr),
		Err(e) => println!("Error: {:?}", e),
	}
}
