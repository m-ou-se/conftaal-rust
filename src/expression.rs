use std::rc::Rc;

use operator::{UnaryOperator,BinaryOperator,Operator};

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
	Integer{ value: i64 },
	Double{ value: f64 },
	String{ value: &'a str },
	List{ elements: Vec<Rc<Expression<'a>>> },
	Object{
		// TODO: Should be ListLiteralExpressions
		keys: Vec<Rc<Expression<'a>>>,
		values: Vec<Rc<Expression<'a>>>,
	},
}
