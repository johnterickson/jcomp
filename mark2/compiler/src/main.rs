extern crate pom;
#[macro_use]
extern crate lazy_static;

use std::io::{self, Read};
use std::iter::FromIterator;
use std::str::FromStr;

use pom::parser::*;

#[derive(Debug, PartialEq)]
struct Function {
    pub name: String,
    pub parameter_names: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq)]
enum Operator {
    Or,
    Equals,
    Add,
    Subtract,
}

#[derive(Debug, PartialEq)]
enum Expression {
    Local {name: String},
    Number {value: u8 },
    Pair {op: Operator, left: Box<Expression>, right: Box<Expression>},
}

#[derive(Debug, PartialEq)]
enum Statement {
    LocalDeclaration { name: String },
    Assignment { name: String, value: Expression },
    If {condition: Expression, true_block: Vec<Statement>, false_block: Vec<Statement>},
    Return { name: String }
}

fn as_chars(s: &str) -> Vec<char> {
    let chars : Vec<char> = s.chars().collect();
    chars
}

fn spaces<'a>() -> Parser<'a, char, ()> {
    one_of(" \t").repeat(1..).discard()
}

fn new_line<'a>() -> Parser<'a, char, ()> {
    one_of("\r\n").discard()
}

fn name<'a>() -> Parser<'a, char, String> {
    none_of(" \r\n\t").repeat(1..).map(String::from_iter)
}

fn operator<'a>() -> Parser<'a, char, Operator> {
    one_of("|=+-").convert(|o| match o {
        '|' => Ok(Operator::Or),
        '=' => Ok(Operator::Equals),
        '+' => Ok(Operator::Add),
        '-' => Ok(Operator::Subtract),
        _ => Err(())
    })
}

fn expression_local<'a>() -> Parser<'a, char, Expression> {
    none_of(" \r\n\t())").repeat(1..).map(String::from_iter)
    .map(|name| Expression::Local { name })
}

fn expression_number<'a>() -> Parser<'a, char, Expression> {
    one_of("0123456789").repeat(1..).collect()
    .convert(|s| u8::from_str(&String::from_iter(s)))
    .map(|value| Expression::Number { value })
}

fn expression_pair<'a>() -> Parser<'a, char, Expression> {
    (
      (sym('(') - sym(' ')) * operator()
    + (sym(' ')) * expression()
    + (sym(' ')) * expression() - sym(' ') - sym(')')
    ).map(|((op, left), right)| Expression::Pair { op, left: Box::new(left), right: Box::new(right) })
}

fn expression<'a>() -> Parser<'a, char, Expression> {
    //  expression_pair() 
      expression_number()
    | expression_local()
    | expression_pair() 
}

lazy_static! {
    static ref DECLARE : Vec<char> = as_chars("DECLARE");
    static ref RETURN : Vec<char> = as_chars("RETURN");
    static ref ASSIGN : Vec<char> = as_chars("ASSIGN");
    static ref BEGIN : Vec<char> = as_chars("BEGIN");
    static ref END : Vec<char> = as_chars("END");
}

fn declare_local<'a>() -> Parser<'a, char, Statement> {
    let stmt = (seq(&DECLARE) - sym(' ')) * name();
    stmt.map(|name| Statement::LocalDeclaration { name })
}

fn if_statement<'a>() -> Parser<'a, char, Statement> {
    // let stmt = 
    //     (seq(b"IF") - space_on_line()) * expression() 
    //   - (space_on_line() * seq(b"THEN") * new_line())
    //   + (space_on_line() * list(call(statement), new_line())
    // stmt.map(|(condition, true_block, false_block)| Statement::If {condition, true_block, false_block})
    unimplemented!();
}

fn return_statement<'a>() -> Parser<'a, char, Statement> {
    let stmt = (seq(&RETURN) - sym(' ')) * name();
    stmt.map(|name| Statement::Return {name})
}

fn assignment<'a>() -> Parser<'a, char, Statement> {
    let stmt = (seq(&ASSIGN) - sym(' ')) * name() - sym(' ') + expression();
    stmt.map(|(name, value)| Statement::Assignment {name, value})
}

fn statement<'a>() -> Parser<'a, char, Statement> {
    // (declare_local() | assignment() | if_statement() | return_statement()) - new_line()
    declare_local() | return_statement() | assignment()
}

fn function<'a>() -> Parser<'a, char, Function> {
    let f = 
          ((seq(&BEGIN) - sym(' ')) * name())
        + ((spaces() * name()).repeat(0..) - new_line())
        + ((sym(' ') * statement() - new_line()).repeat(1..))
        - seq(&END) - new_line();
    f.map(|((name,parameter_names), body)| Function {name, parameter_names, body})
}

fn program<'a>() -> Parser<'a, char, Vec<Function>> {
    let funcs = function().repeat(1..);
    funcs
}

fn main() -> Result<(), std::io::Error> {
    let chars = {
        let mut s = String::new();
        let stdin = io::stdin();
        stdin.lock().read_to_string(&mut s)?;
        let chars : Vec<char> = s.chars().collect();
        chars
    };

    let parser = function();

    println!("{:?}", parser.parse(&chars));

    Ok(())
}
