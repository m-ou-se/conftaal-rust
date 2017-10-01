mod consume;
mod error;
mod whitespace;
pub mod matcher; // TODO: make private

use std::mem;
use std::rc::Rc;

extern crate stringpool;
use self::stringpool::stringtracker::StringTracker;

use expression::{Expression,OpAndLhs,Literal};
use operator::{UnaryOperator,BinaryOperator,Operator,Order,higher_precedence};
use self::consume::Consume;
use self::error::{Error, Message, error};
use self::matcher::{End, OptionalEnd};
use self::whitespace::skip_whitespace;

pub struct Parser<'a> {
	//string_tracker: StringTracker<'a>,
	pub source: &'a str,
}

impl<'a> Parser<'a> {

	pub fn parse_list(&mut self, end: &End<'a>) -> Result<Vec<Rc<Expression<'a>>>, Error<'a>> {
		let mut elements = Vec::new();
		let element_end = End::Specific(",").or_before(*end);
		loop {
			if end.parse(&mut self.source)? { return Ok(elements); }
			elements.push(Rc::new(self.parse_expression(&element_end)?));
		}
	}

	pub fn parse_object(&mut self, end: &End<'a>)
		-> Result<(Vec<Rc<Expression<'a>>>, Vec<Rc<Expression<'a>>>), Error<'a>>
	{
		let mut keys = Vec::new();
		let mut values = Vec::new();
		let element_end = End::ElementEnd.or_before(*end);
		loop {
			if end.parse(&mut self.source)? { return Ok((keys, values)); }
			let key = self.parse_identifier().ok_or_else(||
				error(&self.source[..0], "expected identifier as object key".to_string())
			)?;
			skip_whitespace(&mut self.source, false);
			self.source.consume("=").ok_or_else(||
				error(&self.source[..0], "expected `='".to_string())
			)?;
			let value = self.parse_expression(&element_end)?;
			keys.push(Rc::new(Expression::Identifier(key)));
			values.push(Rc::new(value));
		}
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

	pub fn parse_expression(&mut self, end: &OptionalEnd<'a>) -> Result<Expression<'a>, Error<'a>> {
		let mut expr = self.parse_expression_atom(end)?.ok_or_else(||
			error(&self.source[..0], "missing expression".to_string())
		)?;
		while self.parse_more_expression(&mut expr, end)? {}
		Ok(expr)
	}

	fn parse_expression_atom<'b>(&mut self, end: &OptionalEnd<'a>) -> Result<Option<Expression<'a>>, Error<'a>> {

		if end.parse(&mut self.source)? { return Ok(None); }

		if let Some(open) = self.source.consume("(") {
			let mut expr = self.parse_expression(&End::MatchingBracket(open, ")").as_optional())?;
			if let &mut Expression::Op{ref mut parenthesized, ..} = &mut expr {
				*parenthesized = true;
			}
			Ok(Some(expr))

		} else if let Some((op_source, op)) = self.parse_unary_operator() {
			match self.parse_expression_atom(end)? {
				None => Err(error(&self.source[..0], format!("missing expression after unary `{}' operator", op_source))),
				Some(subexpr) => Ok(Some(Expression::Op{
					op_source: op_source,
					op_and_lhs: OpAndLhs::UnaryOp{op: op},
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
			let (keys, values) = self.parse_object(&End::MatchingBracket(open, "}"))?;
			Ok(Some(Expression::Literal(Literal::Object(keys, values))))

		} else if let Some(open) = self.source.consume("[") {
			let list = self.parse_list(&End::MatchingBracket(open, "]"))?;
			Ok(Some(Expression::Literal(Literal::List(list))))

		} else if let Some(number) = self.parse_number() {
			Ok(Some(Expression::Literal(number)))

		} else if self.source.starts_with("\\") {
			// TODO: parse lambda
			unimplemented!();

		} else {
			Ok(None)
		}
	}

	fn parse_more_expression(&mut self, expr: &mut Expression<'a>, end: &OptionalEnd<'a>) -> Result<bool, Error<'a>> {

		if end.parse(&mut self.source)? { return Ok(false); }

		let (op_source, op) = self.parse_binary_operator().ok_or_else(||
			error(&self.source[..0], format!("expected binary operator or {}", end.description()))
		)?;

		let rhs = match op {
			BinaryOperator::Call  => Expression::Literal(Literal::List(self.parse_list(&End::MatchingBracket(op_source, ")"))?)),
			BinaryOperator::Index => Expression::Literal(Literal::List(self.parse_list(&End::MatchingBracket(op_source, "]"))?)),
			BinaryOperator::Dot => {
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

		*old_lhs = Expression::Op{
			op_source: op_source,
			op_and_lhs: OpAndLhs::BinaryOp{
				op: op,
				lhs: new_lhs,
			},
			rhs: Rc::new(rhs),
			parenthesized: false,
		};

		Ok(true)
	}

	fn parse_unary_operator(&mut self) -> Option<(&'a str, UnaryOperator)> {
		use self::UnaryOperator::*;
		match self.source.as_bytes().get(0) {
			Some(&b'+') => Some(Plus),
			Some(&b'-') => Some(Minus),
			Some(&b'!') => Some(Complement),
			Some(&b'~') => Some(LogicalNot),
			_ => None,
		}.map(|op| (self.source.consume_n(1), op))
	}

	fn parse_binary_operator(&mut self) -> Option<(&'a str, BinaryOperator)> {
		use self::BinaryOperator::*;
		let b: &[u8] = self.source.as_bytes();
		match (
			match b.get(0) { Some(&x) => x, None => 0 },
			match b.get(1) { Some(&x) => x, None => 0 },
		) {
			(b'.',    _) => Some((1, Dot           )),
			(b'[',    _) => Some((1, Index         )),
			(b'(',    _) => Some((1, Call          )),
			(b':',    _) => Some((1, Colon         )),
			(b'*', b'*') => Some((2, Power         )),
			(b'*',    _) => Some((1, Times         )),
			(b'/',    _) => Some((1, Divide        )),
			(b'%',    _) => Some((1, Modulo        )),
			(b'+',    _) => Some((1, Plus          )),
			(b'-',    _) => Some((1, Minus         )),
			(b'<', b'=') => Some((2, LessOrEqual   )),
			(b'<', b'<') => Some((2, LeftShift     )),
			(b'<',    _) => Some((1, Less          )),
			(b'>', b'=') => Some((2, GreaterOrEqual)),
			(b'>', b'>') => Some((2, RightShift    )),
			(b'>',    _) => Some((1, Greater       )),
			(b'=', b'=') => Some((2, Equal         )),
			(b'!', b'=') => Some((2, Inequal       )),
			(b'&', b'&') => Some((2, LogicalAnd    )),
			(b'&',    _) => Some((1, BitAnd        )),
			(b'^',    _) => Some((1, BitXor        )),
			(b'|', b'|') => Some((2, LogicalOr     )),
			(b'|',    _) => Some((1, BitOr         )),
			_ => None,
		}.map(|(n, op)| (self.source.consume_n(n), op))
	}

	fn parse_string_literal(&mut self) -> Literal<'a> {
		unimplemented!()
	}

	fn parse_number(&mut self) -> Option<Literal<'a>> {
		unimplemented!()
	}

}

fn find_lhs<'a, 'b>(
	op: BinaryOperator,
	op_source: &'a str,
	mut expr: &'b mut Expression<'a>
) -> Result<&'b mut Expression<'a>, Error<'a>> {
	loop {
		let current = expr;
		match current {
			&mut Expression::Op{
				op_and_lhs: ref e_op_and_lhs,
				op_source: e_op_source,
				rhs: ref mut e_rhs,
				parenthesized: false,
				..
			} if !is_lhs(e_op_and_lhs.op(), e_op_source, op, op_source)? => {
				expr = Rc::get_mut(e_rhs).unwrap();
			}
			_ => return Ok(current)
		}
	}
}

fn is_lhs<'a>(
	left_op: Operator,
	left_op_source: &'a str,
	op: BinaryOperator,
	op_source: &'a str
) -> Result<bool, Error<'a>> {
	match higher_precedence(left_op, Operator::Binary(op)) {
		Order::Left => Ok(true),
		Order::Right => Ok(false),
		Order::Unordered => Err(Error{
			message: Message{
				message: if Operator::Binary(op) == left_op {
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
