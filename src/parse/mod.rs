mod consume;
mod error;
mod hex;
mod matcher;

use std::rc::Rc;
use std::mem;

extern crate stringpool;
use self::stringpool::stringtracker::StringTracker;

use expression::*;
use operator::{Operator,Order,higher_precedence};
use self::consume::Consume;
use self::matcher::*;

struct Parser<'a> {
	string_tracker: StringTracker<'a>,
	source: &'a str,
}

impl<'a> Parser<'a> {
	pub fn parse_expression(&mut self /*, end */) -> Result<Expression<'a>, ()> {
		let mut expr = self.parse_expression_atom(/* end */)?.ok_or_else(||
			() // TODO: Missing expression
		)?;
		while self.parse_more_expression(&mut expr)? {}
		Ok(expr)
	}

	pub fn parse_list(&mut self /*, end */) -> Expression<'a> {
		unimplemented!();
	}

	pub fn parse_object(&mut self /*, end */) -> Expression<'a> {
		unimplemented!();
	}

	pub fn parse_identifier(&mut self) -> Option<&'a str> {
		if self.source.starts_with(|c|
			(c >= 'a' && c <= 'z') ||
			(c >= 'A' && c <= 'Z') ||
			c == '_'
		) {
			Some(self.source.consume_while(|c|
				(c >= 'a' && c <= 'z') ||
				(c >= 'A' && c <= 'Z') ||
				(c >= '0' && c <= '9') ||
				c == '_'
			))
		} else {
			None
		}
	}

//private:
	fn parse_expression_atom(&mut self /*, end */) -> Result<Option<Expression<'a>>, ()> {
		//TODO: if parse_end(..) return None

		if let Some(open) = self.source.consume("(") {
			let mut expr = self.parse_expression(/*Matcher(MatchMode::MatchingBracket, ")", open)*/)?;
			if let &mut Expression::Operator{ref mut parenthesized, ..} = &mut expr {
				*parenthesized = true;
			}
			Ok(Some(expr))

		} else if let Some(op_source) = self.source.consume_if(|c| "!~-+".contains(c)) {
			match self.parse_expression_atom(/* end */)? {
				None => Err(()), // TODO: "missing expression after unary `{}' operator"
				Some(subexpr) => Ok(Some(Expression::Operator{
					op: Operator::Plus, // TODO
					op_source: op_source,
					lhs: None,
					rhs: Rc::new(subexpr),
					parenthesized: false
				}))
			}

		} else if let Some(identifier) = self.parse_identifier() {
			Ok(Some(Expression::Identifier(identifier)))

		} else if self.source.starts_with("\"") {
			// TODO: parse string literal
			unimplemented!();

		} else if let Some(open) = self.source.consume("{") {
			// TODO: parse object
			unimplemented!();

		} else if let Some(open) = self.source.consume("[") {
			// TODO: parse list
			unimplemented!();

		} else if let Some(number) = self.parse_number() {
			Ok(Some(Expression::Literal(number)))

		} else if self.source.starts_with("\\") {
			// TODO: parse lambda
			unimplemented!();

		} else {
			Ok(None)
		}
	}

	fn parse_more_expression(&mut self, expr: &mut Expression<'a> /*, end*/) -> Result<bool, ()> {
		// TODO: if parse end, return false

		let (op_source, op) = self.parse_binary_operator().ok_or_else(||
			() // TODO: Expected binary operator or end.description()
		)?;

		let rhs = match op {
			Operator::Index | Operator::Call => {
				self.parse_list(/* TODO: end */)
			},
			Operator::Dot => {
				self.parse_identifier().map(|ident|
					Expression::Identifier(ident)
				).ok_or_else(||
					() // TODO: expected identifier after .
				)?
			},
			_ => {
				self.parse_expression_atom(/* end */)?.ok_or_else(||
					() // TODO: missing expression after binop
				)?
			},
		};

		let old_lhs: &mut Expression<'a> = find_lhs(op, expr)?;

		// Use a dummy value of Identifier("") while we swap the nodes around.
		let new_lhs = Rc::new(mem::replace(old_lhs, Expression::Identifier("")));

		*old_lhs = Expression::Operator{
			op: op,
			op_source: op_source,
			lhs: Some(new_lhs),
			rhs: Rc::new(rhs),
			parenthesized: false,
		};

		Ok(true)
	}

	fn parse_binary_operator(&mut self) -> Option<(&'a str, Operator)> {
		let b: &[u8] = self.source.as_bytes();
		match (
			match b.get(0) { Some(&x) => x, None => 0 },
			match b.get(1) { Some(&x) => x, None => 0 },
		) {
			(b'.',    _) => Some((1, Operator::Dot           )),
			(b'[',    _) => Some((1, Operator::Index         )),
			(b'(',    _) => Some((1, Operator::Call          )),
			(b':',    _) => Some((1, Operator::Colon         )),
			(b'*', b'*') => Some((2, Operator::Power         )),
			(b'*',    _) => Some((1, Operator::Times         )),
			(b'/',    _) => Some((1, Operator::Divide        )),
			(b'%',    _) => Some((1, Operator::Modulo        )),
			(b'+',    _) => Some((1, Operator::Plus          )),
			(b'-',    _) => Some((1, Operator::Minus         )),
			(b'<', b'=') => Some((2, Operator::LessOrEqual   )),
			(b'<', b'<') => Some((2, Operator::LeftShift     )),
			(b'<',    _) => Some((1, Operator::Less          )),
			(b'>', b'=') => Some((2, Operator::GreaterOrEqual)),
			(b'>', b'>') => Some((2, Operator::RightShift    )),
			(b'>',    _) => Some((1, Operator::Greater       )),
			(b'=', b'=') => Some((2, Operator::Equal         )),
			(b'!', b'=') => Some((2, Operator::Inequal       )),
			(b'&', b'&') => Some((2, Operator::LogicalAnd    )),
			(b'&',    _) => Some((1, Operator::BitAnd        )),
			(b'^',    _) => Some((1, Operator::BitXor        )),
			(b'|', b'|') => Some((2, Operator::LogicalOr     )),
			(b'|',    _) => Some((1, Operator::BitOr         )),
			_ => None,
		}.map(|(n, op)| (self.source.consume_n(n), op))
	}

	fn parse_string_literal(&mut self) -> Literal<'a> {
		unimplemented!()
	}

	fn parse_number(&mut self) -> Option<Literal<'a>> {
		unimplemented!()
	}

	//optional<string_view> parse_end(Matcher const &, bool consume = true);

}

fn find_lhs<'a, 'b>(op: Operator, expr: &'b mut Expression<'a>) -> Result<&'b mut Expression<'a>, ()> {
	match expr {
		&mut Expression::Operator{
			op: e_op,
			rhs: ref mut e_rhs,
			parenthesized: false,
			..
		} if match higher_precedence(e_op, op) {
			Order::Left => false,
			Order::Right => true,
			Order::Unordered => {
				return Err(()); // TODO:
				// Operator X [has equal precedence as Y and ]is non-associative
				// Note: Conflicting Y here
			},
		} => find_lhs(op, Rc::get_mut(e_rhs).unwrap()),
		_ => Ok(expr),
	}
}
