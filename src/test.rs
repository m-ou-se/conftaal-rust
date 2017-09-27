extern crate conftaal;

use conftaal::expression::{Expression,OpAndLhs,Literal};
use conftaal::parse::Parser;
use conftaal::parse::matcher::{Matcher, MatchMode};

fn reconstruct(e: &Expression) -> String {
	use Expression::*;
	use OpAndLhs::*;
	use Literal::*;
	match *e {
		Identifier(a) => a.to_string(),
		Op{op_source, ref op_and_lhs, ref rhs, parenthesized} => {
			let mut s = match *op_and_lhs {
				UnaryOp{..} => format!("{}{}", op_source, reconstruct(rhs.as_ref())),
				BinaryOp{ref lhs, ..} => format!("{} {} {}", reconstruct(lhs.as_ref()), op_source, reconstruct(rhs.as_ref())),
			};
			if parenthesized { s = format!("({})", s); }
			s
		},
		Literal(List{ref elements}) => {
			let mut s = "list(".to_string();
			for (i, e) in elements.iter().enumerate() {
				if i > 0 { s += ", "; }
				s += &reconstruct(e)[..];
			}
			s += ")";
			s
		}
		Literal(_) => "<some literal>".to_string(),
	}
}

fn main() {
	let mut p = Parser{
		source: "a[x, (w>a)(a, b+c), z + q]" //"a * (-a.bla + b)"
	};

	println!("Parsing: {}", p.source);

	match p.parse_expression(&Matcher::new(MatchMode::EndOfFile)) {
		Ok(expr) => {
			println!("Result: {:?}", expr);
			println!("Reconstucted: {}", reconstruct(&expr));
		},
		Err(e) => println!("Error: {:?}", e),
	}
}
