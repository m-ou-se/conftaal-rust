mod consume;
mod error;
mod whitespace;
pub mod end; // TODO: make private

use std::mem;
use std::rc::Rc;
use std::u64;

use expression::{Expression,Op,Literal};
use operator::{UnaryOperator,BinaryOperator,Operator,Order,higher_precedence};
use self::consume::Consume;
use self::end::{End, OptionalEnd};
use self::error::{Error, Message, error};
use self::whitespace::skip_whitespace;

pub struct Parser<'a> {
	pub source: &'a [u8],
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
			keys.push(Rc::new(Expression::Literal(Literal::String(key))));
			values.push(Rc::new(value));
		}
	}

	pub fn parse_identifier(&mut self) -> Option<&'a str> {
		fn is_identifier_char(c: char, start: bool) -> bool {
			match c {
				'a'...'z' | 'A'...'Z' | '_' => true,
				'0'...'9' => !start,
				_ => false,
			}
		}
		self.source.first().and_then(|&b|
			if is_identifier_char(b as char, true) {
				Some(self.source.consume_while(|c| is_identifier_char(c, false)))
			} else {
				None
			}
		)
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
					op: Op::UnaryOp{op, rhs: Rc::new(subexpr)},
					parenthesized: false
				}))
			}

		} else if let Some(identifier) = self.parse_identifier() {
			Ok(Some(Expression::Identifier(identifier)))

		} else if self.source.starts_with(b"\"") {
			unimplemented!("string literals");

		} else if let Some(open) = self.source.consume("{") {
			let (keys, values) = self.parse_object(&End::MatchingBracket(open, "}"))?;
			Ok(Some(Expression::Literal(Literal::Object(keys, values))))

		} else if let Some(open) = self.source.consume("[") {
			let list = self.parse_list(&End::MatchingBracket(open, "]"))?;
			Ok(Some(Expression::Literal(Literal::List(list))))

		} else if self.source.starts_with(b"\"") {
			Ok(Some(Expression::Literal(self.parse_string_literal())))

		} else if let Some(number) = self.parse_number()? {
			Ok(Some(Expression::Literal(number)))

		} else if self.source.starts_with(b"\\") {
			unimplemented!("lambdas");

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
			op: Op::BinaryOp{
				op: op,
				lhs: new_lhs,
				rhs: Rc::new(rhs),
			},
			parenthesized: false,
		};

		Ok(true)
	}

	fn parse_unary_operator(&mut self) -> Option<(&'a str, UnaryOperator)> {
		use self::UnaryOperator::*;
		self.source.first().and_then(|x| match x {
			b'+' => Some(Plus),
			b'-' => Some(Minus),
			b'!' => Some(Complement),
			b'~' => Some(LogicalNot),
			_ => None,
		}).map(|op| (unsafe { self.source.consume_str_n(1) }, op))
	}

	fn parse_binary_operator(&mut self) -> Option<(&'a str, BinaryOperator)> {
		use self::BinaryOperator::*;
		if let Some(op) = self.source.get(0..2).and_then(|x| match x {
			b"**" => Some(Power         ),
			b"<=" => Some(LessOrEqual   ),
			b"<<" => Some(LeftShift     ),
			b">=" => Some(GreaterOrEqual),
			b">>" => Some(RightShift    ),
			b"==" => Some(Equal         ),
			b"!=" => Some(Inequal       ),
			b"&&" => Some(LogicalAnd    ),
			b"||" => Some(LogicalOr     ),
			_ => None,
		}) {
			return Some((unsafe { self.source.consume_str_n(2) }, op));
		}
		if let Some(op) = self.source.get(0).and_then(|x| match x {
			b'.' => Some(Dot    ),
			b'[' => Some(Index  ),
			b'(' => Some(Call   ),
			b':' => Some(Colon  ),
			b'*' => Some(Times  ),
			b'/' => Some(Divide ),
			b'%' => Some(Modulo ),
			b'+' => Some(Plus   ),
			b'-' => Some(Minus  ),
			b'<' => Some(Less   ),
			b'>' => Some(Greater),
			b'&' => Some(BitAnd ),
			b'^' => Some(BitXor ),
			b'|' => Some(BitOr  ),
			_ => None,
		}) {
			return Some((unsafe { self.source.consume_str_n(1) }, op));
		}
		None
	}

	fn parse_string_literal(&mut self) -> Literal<'a> {
		unimplemented!("string literals");
	}

	fn parse_number(&mut self) -> Result<Option<Literal<'a>>, Error<'a>> {
		let s = &mut self.source;

		if s.get(s.starts_with(b".") as usize).map(|&b| (b as char).is_digit(10)) != Some(true) {
			return Ok(None);
		}

		let base = if let Some(b) = s.get(0..2).and_then(|x| match x {
			b"0x" | b"0X" => Some(16),
			b"0o" | b"0O" => Some(8),
			b"0b" | b"0B" => Some(2),
			_ => None
		}) {
			s.consume_n(2);
			b
		} else {
			10
		};

		let integer_part = s.consume_while(|c| c.is_digit(base));

		let fractional_part = s.consume(".").map(|_| s.consume_while(|c| c.is_digit(base)));

		let exponent_part = s.consume_one_of(if base == 16 {"pP"} else {"eE"}).map(|_| (
			s.consume_one_of("+-") == Some("-"),
			s.consume_while(|c| c.is_digit(base))
		));

		if exponent_part.is_none() && fractional_part.is_none() {
			// Integer
			if integer_part.is_empty() {
				return Err(error(integer_part.as_bytes(), "missing digits".to_string()));
			}
			match u64::from_str_radix(integer_part, base) {
				Ok(i) => Ok(Some(Literal::Integer(i))),
				Err(_) => Err(error(integer_part.as_bytes(), "integer too large".to_string())),
			}
		} else {
			// Float
			unimplemented!("parsing float literals");
		}

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
			Expression::Op{
				op: e_op,
				op_source: e_op_source,
				parenthesized: false,
				..
			} if !is_lhs(e_op.op(), e_op_source, op, op_source)? => {
				expr = Rc::get_mut(match e_op {
					Op::UnaryOp{rhs, ..} | Op::BinaryOp{rhs, ..} => rhs
				}).unwrap();
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
				location: Some(op_source.as_bytes()),
			},
			notes: vec![Message{
				message: format!("conflicting `{}' here", left_op_source),
				location: Some(left_op_source.as_bytes()),
			}],
		})
	}
}
