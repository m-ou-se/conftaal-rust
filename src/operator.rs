#[derive(Clone, Copy)]
pub enum Operator {
	/* .  */ Dot,
	/* [] */ Index,
	/* () */ Call,
	/* :  */ Colon,
	/* == */ Equal,
	/* != */ Inequal,
	/* >  */ Greater,
	/* <  */ Less,
	/* >= */ GreaterOrEqual,
	/* <= */ LessOrEqual,
	/* +  */ UnaryPlus,
	/* -  */ UnaryMinus,
	/* ~  */ Complement,
	/* !  */ LogicalNot,
	/* +  */ Plus,
	/* -  */ Minus,
	/* *  */ Times,
	/* /  */ Divide,
	/* %  */ Modulo,
	/* ** */ Power,
	/* << */ LeftShift,
	/* >> */ RightShift,
	/* &  */ BitAnd,
	/* |  */ BitOr,
	/* ^  */ BitXor,
	/* && */ LogicalAnd,
	/* || */ LogicalOr
}

#[derive(Clone, Copy)]
pub enum Order {
	Left,
	Right,
	Unordered
}

// A lower value means a higher precedence.
fn get_precedence(op: Operator) -> i32 {
	use operator::Operator::*;
	match op {
		Dot             |
		Colon           |
		Call            |
		Index           => 1,
		UnaryPlus       |
		UnaryMinus      |
		Complement      |
		LogicalNot      => 3,
		Power           => 4,
		Times           |
		Divide          |
		Modulo          => 5,
		Plus            |
		Minus           => 6,
		LeftShift       |
		RightShift      => 7,
		Greater         |
		Less            |
		GreaterOrEqual  |
		LessOrEqual     => 8,
		Equal           |
		Inequal         => 9,
		BitAnd          => 10,
		BitXor          => 11,
		BitOr           => 12,
		LogicalAnd      => 13,
		LogicalOr       => 14,
	}
}

fn get_associativity(precedence: i32) -> Order {
	match precedence {
		4     => Order::Right,     // **
		8 | 9 => Order::Unordered, // > < >= <= != ==
		_     => Order::Left,
	}
}

pub fn higher_precedence(left_op: Operator, right_op: Operator) -> Order {
	let left = get_precedence(left_op);
	let right = get_precedence(right_op);
	if left < right {
		Order::Left
	} else if left > right {
		Order::Right
	} else {
		get_associativity(left)
	}
}
