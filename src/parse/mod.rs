mod consume;
mod error;
mod hex;
pub mod matcher; // TODO: make private

use std::mem;
use std::rc::Rc;

extern crate stringpool;
use self::stringpool::stringtracker::StringTracker;

use expression::*;
use operator::{Operator,Order,higher_precedence};
use self::consume::Consume;
use self::error::{Error, Message};
use self::matcher::Matcher;

pub struct Parser<'a> {
	//string_tracker: StringTracker<'a>,
	pub source: &'a str,
}

fn error<'a>(location: &'a str, message: String) -> Error<'a> {
	Error{
		message: Message{
			message: message,
			location: Some(location),
		},
		notes: vec![],
	}
}

impl<'a> Parser<'a> {

	pub fn parse_list(&mut self, end: &Matcher<'a>) -> Expression<'a> {
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

	pub fn parse_expression(&mut self, end: &Matcher<'a>) -> Result<Expression<'a>, Error<'a>> {
		let mut expr = self.parse_expression_atom(end)?.ok_or_else(||
			error(&self.source[..0], "missing expression".to_string())
		)?;
		while self.parse_more_expression(&mut expr, end)? {}
		Ok(expr)
	}

	fn parse_expression_atom(&mut self, end: &Matcher<'a>) -> Result<Option<Expression<'a>>, Error<'a>> {

		if end.parse_end(&mut self.source)? { return Ok(None); }

		if let Some(open) = self.source.consume("(") {
			let mut expr = self.parse_expression(&Matcher::bracket(open, ")"))?;
			if let &mut Expression::Operator{ref mut parenthesized, ..} = &mut expr {
				*parenthesized = true;
			}
			Ok(Some(expr))

		} else if let Some((op_source, op)) = self.parse_unary_operator() {
			match self.parse_expression_atom(end)? {
				None => Err(error(&self.source[..0], format!("missing expression after unary `{}' operator", op_source))),
				Some(subexpr) => Ok(Some(Expression::Operator{
					op: op,
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

	fn parse_more_expression(&mut self, expr: &mut Expression<'a>, end: &Matcher<'a>) -> Result<bool, Error<'a>> {

		if end.parse_end(&mut self.source)? { return Ok(false); }

		let (op_source, op) = self.parse_binary_operator().ok_or_else(||
			error(&self.source[..0], format!("expected binary operator or {}", "TODO end.description()"))
		)?;

		let rhs = match op {
			Operator::Call  => self.parse_list(&Matcher::bracket(op_source, ")")),
			Operator::Index => self.parse_list(&Matcher::bracket(op_source, "]")),
			Operator::Dot => {
				self.parse_identifier().map(|ident|
					Expression::Identifier(ident)
				).ok_or_else(||
					error(&self.source[..0], "expected identifier after `.'".to_string())
				)?
			},
			_ => {
				self.parse_expression_atom(end)?.ok_or_else(||
					error(&self.source[..0], format!("expected expression after binary operator `{}'", op_source))
				)?
			},
		};

		let old_lhs: &mut Expression<'a> = find_lhs(op, op_source, expr)?;

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

	fn parse_unary_operator(&mut self) -> Option<(&'a str, Operator)> {
		match self.source.as_bytes().get(0) {
			Some(&b'+') => Some(Operator::UnaryPlus),
			Some(&b'-') => Some(Operator::UnaryMinus),
			Some(&b'!') => Some(Operator::Complement),
			Some(&b'~') => Some(Operator::LogicalNot),
			_ => None,
		}.map(|op| (self.source.consume_n(1), op))
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

fn find_lhs<'a, 'b>(op: Operator, op_source: &'a str, mut expr: &'b mut Expression<'a>) -> Result<&'b mut Expression<'a>, Error<'a>> {
	loop {
		let current = expr;
		match current {
			&mut Expression::Operator{
				op: e_op,
				op_source: e_op_source,
				rhs: ref mut e_rhs,
				parenthesized: false,
				..
			} if !is_lhs(e_op, e_op_source, op, op_source)? => {
				expr = Rc::get_mut(e_rhs).unwrap();
			}
			_ => return Ok(current)
		}
	}
}

fn is_lhs<'a>(left_op: Operator, left_op_source: &'a str, op: Operator, op_source: &'a str) -> Result<bool, Error<'a>> {
	match higher_precedence(left_op, op) {
		Order::Left => Ok(true),
		Order::Right => Ok(false),
		Order::Unordered => Err(Error{
			message: Message{
				message: if op == left_op {
						format!("operator `{}' is non-associative", op_source)
					} else {
						format!("operator `{}' has equal precedence as `{}' and is non-associative", op_source, left_op_source)
					},
				location: Some(op_source),
			},
			notes: vec![Message{
				message: format!("conflicting `{}' here", left_op_source),
				location: Some(left_op_source),
			}],
		})
	}
}
