use std::rc::Rc;

use operator::{UnaryOperator, BinaryOperator, Operator};

#[derive(Debug)]
pub enum Expression<'a> {
	Identifier(&'a str),
	Op{
		op_source: &'a str,
		op: Op<'a>,
		parenthesized: bool,
	},
	Literal(Literal<'a>),
}

#[derive(Debug)]
pub enum Op<'a> {
	UnaryOp{
		op: UnaryOperator,
		rhs: Rc<Expression<'a>>,
	},
	BinaryOp{
		op: BinaryOperator,
		rhs: Rc<Expression<'a>>,
		lhs: Rc<Expression<'a>>,
	},
}

impl<'a> Op<'a> {
	pub fn op(&self) -> Operator {
		match self {
			&Op::UnaryOp{op, ..} => Operator::Unary(op),
			&Op::BinaryOp{op, ..} => Operator::Binary(op),
		}
	}
}

#[derive(Debug)]
pub enum Literal<'a> {
	Integer(u64),
	Double(f64),
	String(&'a str),
	List(Vec<Rc<Expression<'a>>>),
	Object(Vec<Rc<Expression<'a>>>, Vec<Rc<Expression<'a>>>),
}
