extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

use std::io;
use std::io::ErrorKind;
use std::io::Read;
use std::collections::{BTreeMap,BTreeSet};
use std::str::FromStr;

#[derive(Parser)]
#[grammar = "j.pest"]
struct ProgramParser;

#[derive(Debug, PartialEq)]
enum Operator {
    Add,
    Subtract,
    Multiply,
    Or,
    Equals
}

impl Operator {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Operator {
        match pair.as_str() {
            "+" => Operator::Add,
            "-" => Operator::Subtract,
            "*" => Operator::Multiply,
            "||" => Operator::Or,
            "==" => Operator::Equals,
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
enum Expression {
    Ident(String),
    Number(i32),
    Operation(Operator, Box<Expression>, Box<Expression>)
}

impl Expression {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Expression {
        assert_eq!(Rule::expression, pair.as_rule());
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::number => {
                let number = pair.into_inner().next().unwrap();
                let number = i32::from_str(number.as_str()).expect("Couldn't parse integer.");
                Expression::Number(number)
            },
            Rule::ident => {
                let label = pair.into_inner().next().unwrap();
                let label = label.as_str().to_owned();
                Expression::Ident(label)
            },
            Rule::operator_expression => {
                let mut pairs = pair.into_inner();
                let left = Expression::parse(pairs.next().unwrap());
                let op = Operator::parse(pairs.next().unwrap());
                let right = Expression::parse(pairs.next().unwrap());
                Expression::Operation(op, Box::new(left), Box::new(right))
            },
            _ => unimplemented!()
        }
        
    }
}

#[derive(Debug)]
enum Statement {
    Assign {local: String, value: Expression},
    Call { local: String, function: String, parameters: Vec<Expression> },
    If {predicate: Expression, when_true: Vec<Statement> },
    Return { local: String},
}

impl Statement {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Statement {
        assert_eq!(Rule::statement, pair.as_rule());
        let pair = pair.into_inner().next().unwrap();

        match pair.as_rule() {
            Rule::assign => {
                let mut pairs = pair.into_inner();
                let local = pairs.next().unwrap().as_str().to_owned();
                let value = Expression::parse(pairs.next().unwrap());
                Statement::Assign { local, value }
            },
            Rule::call => {
                let mut pairs = pair.into_inner();
                let local = pairs.next().unwrap().as_str().to_owned();
                let function = pairs.next().unwrap().as_str().to_owned();

                let mut parameters = Vec::new();
                while let Some(arg) = pairs.next() {
                    parameters.push(Expression::parse(arg));
                }

                Statement::Call { local, function, parameters }
            },
            Rule::if_statement => {
                let mut pairs = pair.into_inner();
                let predicate = Expression::parse(pairs.next().unwrap());
                let mut when_true = Vec::new();
                while let Some(stmt) = pairs.next() {
                    when_true.push(Statement::parse(stmt));
                }
                Statement::If { predicate, when_true }
            },
            Rule::return_statement => {
                let local = pair.into_inner().next().unwrap().as_str().to_owned();
                Statement::Return { local }
            },
            _ => panic!("Unexpected {:?}", pair)
        }
    }
}

#[derive(Debug)]
struct Function {
    name: String,
    args: Vec<String>,
    locals: BTreeSet<String>,
    body: Vec<Statement>,
}

impl Function {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Function {
        assert_eq!(Rule::function, pair.as_rule());

        let mut args = Vec::new();
        let mut locals = BTreeSet::new();

        let mut pairs = pair.into_inner();

        let name = pairs.next().unwrap().as_str().to_owned();
        print!("Function: {}(", &name);

        for arg in pairs.next().unwrap().into_inner() {
            let arg = arg.as_str();
            print!("{},", arg);
            args.push(arg.to_owned());
        }
        println!(")");

        let body : Vec<Statement> = pairs.next().unwrap().into_inner().map(|p| Statement::parse(p)).collect();

        // find locals
        for s in body.iter() {
            match s {
                Statement::Assign{local, value:_} => { locals.insert(local.clone()); },
                Statement::Return{ local:_ } => {},
                Statement::Call{ local, function:_, parameters:_ } => { locals.insert(local.clone()); },
                Statement::If{ predicate:_, when_true:_ } => {},
            }
        }

        println!("Locals:");
        for l in &locals {
            println!(" {}", l);
        }

        Function { name, args, locals, body }
    }
}


fn main() -> Result<(), std::io::Error> {
    let input = {
        let mut s = String::new();
        let stdin = io::stdin();
        stdin.lock().read_to_string(&mut s)?;
        s
    };

    let mut functions = BTreeMap::new();

    let mut program = ProgramParser::parse(Rule::program, &input).unwrap();
    let pairs = program.next().unwrap().into_inner();
    for pair in pairs {
        match pair.as_rule() {
            Rule::function => {
                //ast.push(Print(Box::new(build_ast_from_expr(pair))));
                //println!("Found function: {}\n", pair.as_str());
                let f = Function::parse(pair);
                functions.insert(f.name.clone(), f);
            },
            Rule::EOI => { },
            _ => {
                panic!("Unexpected rule: {:?}", pair);
            }
        }
    }

    let main = functions.get("main");
    if main.is_none() {
        println!("main not found!");
        return Err(std::io::Error::from(ErrorKind::NotFound));
    }

    for f in &functions {
        println!("{:?}", f);
    }

    Ok(())
}
