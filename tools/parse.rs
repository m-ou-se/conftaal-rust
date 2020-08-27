extern crate conftaal;

use std::{env, fs};

use conftaal::expression::{Expression, Literal, Op};
use conftaal::parse::end::End;
use conftaal::parse::Parser;

fn format_list(list: &Vec<Box<Expression>>) -> String {
    let mut s = "(list".to_string();
    for e in list.iter() {
        s += " ";
        s += &format(e)[..];
    }
    s += ")";
    s
}

fn format(e: &Expression) -> String {
    use Expression::*;
    use Literal::*;
    use Op::*;
    match e {
        Identifier(id) => format!("id:{}", id),
        Op {
            op_source,
            op: UnaryOp { rhs, .. },
            ..
        } => format!("(op{} {})", op_source, format(rhs.as_ref())),
        Op {
            op_source,
            op: BinaryOp {
                rhs, lhs, ..
            },
            ..
        } => format!(
            "(op{} {} {})",
            op_source,
            format(lhs.as_ref()),
            format(rhs.as_ref())
        ),
        Literal(List(elements)) => format_list(elements),
        Literal(Object(keys, values)) => format!(
            "(object keys={} values={})",
            format_list(keys),
            format_list(values)
        ),
        Literal(String(s)) => format!("str:{:?}", s),
        Literal(Integer(i)) => format!("int:{}", i),
        Literal(Double(f)) => format!("float:{}", f),
    }
}

fn main() {
    for filename in env::args().skip(1) {
        let source = fs::read(filename).unwrap();

        let mut parser = Parser { source: &source };

        match parser.parse_expression(&End::EndOfFile.as_optional()) {
            Ok(expr) => println!("{}", format(&expr)),
            Err(e) => println!("Error: {:#?}", e),
        };
    }
}
