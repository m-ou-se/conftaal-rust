use std::rc::Rc;

use operator::*;

pub enum Expression<'a> {
	Identifier(&'a str),
	Operator{
		op: Operator,
		op_source: &'a str,
		lhs: Option<Rc<Expression<'a>>>,
		rhs: Rc<Expression<'a>>,
		parenthesized: bool
	},
	Literal(Literal<'a>),
}

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

