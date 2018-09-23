use std::rc::Rc;

use operator::{UnaryOperator, BinaryOperator, Operator};

#[derive(Debug)]
pub enum Expression<'a> {
	Identifier(&'a str),
	Op{
		op_source: &'a str,
		op_and_lhs: OpAndLhs<'a>,
		rhs: Rc<Expression<'a>>,
		parenthesized: bool,
	},
	Literal(Literal<'a>),
}

#[derive(Debug)]
pub enum OpAndLhs<'a> {
	UnaryOp{
		op: UnaryOperator,
	},
	BinaryOp{
		op: BinaryOperator,
		lhs: Rc<Expression<'a>>,
	},
}

impl<'a> OpAndLhs<'a> {
	pub fn op(&self) -> Operator {
		match self {
			&OpAndLhs::UnaryOp{op} => Operator::Unary(op),
			&OpAndLhs::BinaryOp{op, ..} => Operator::Binary(op),
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
