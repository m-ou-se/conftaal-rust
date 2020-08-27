#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnaryOperator {
    /* + */ Plus,
    /* - */ Minus,
    /* ~ */ Complement,
    /* ! */ LogicalNot,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinaryOperator {
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
    /* || */ LogicalOr,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Operator {
    Unary(UnaryOperator),
    Binary(BinaryOperator),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Order {
    Left,
    Right,
    Unordered,
}

// A lower value means a higher precedence.
fn get_precedence(op: Operator) -> i32 {
    use operator::BinaryOperator::*;
    use operator::Operator::*;
    match op {
        Binary(Dot) | Binary(Colon) | Binary(Call) | Binary(Index) => 1,
        Unary(_) => 2,
        Binary(Power) => 3,
        Binary(Times) | Binary(Divide) | Binary(Modulo) => 4,
        Binary(Plus) | Binary(Minus) => 5,
        Binary(LeftShift) | Binary(RightShift) => 6,
        Binary(Greater) | Binary(Less) | Binary(GreaterOrEqual) | Binary(LessOrEqual) => 7,
        Binary(Equal) | Binary(Inequal) => 8,
        Binary(BitAnd) => 9,
        Binary(BitXor) => 10,
        Binary(BitOr) => 11,
        Binary(LogicalAnd) => 12,
        Binary(LogicalOr) => 13,
    }
}

fn get_associativity(precedence: i32) -> Order {
    match precedence {
        3 => Order::Right,         // **
        7 | 8 => Order::Unordered, // > < >= <= != ==
        _ => Order::Left,
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
