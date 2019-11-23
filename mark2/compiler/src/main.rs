extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

use std::io;
use std::io::Read;
use std::collections::BTreeSet;

#[derive(Parser)]
#[grammar = "j.pest"]
struct ProgramParser;

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

enum Expression {
    Ident(String),
    Number(usize),
    Operation(Operator, Box<Expression>, Box<Expression>)
}

enum Statement {
    Assign {local: String, value: Expression},
    Call { local: String, function: String, parameters: Vec<Expression> },
    If {predicate: Expression, when_true: Vec<Statement> },
    Return { local: String},
}

impl Statement {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Statement {
        match pair.as_rule() {
            Rule::assign => {
                let mut pairs = pair.into_inner();
                let local = pairs.next().unwrap().as_str().to_owned();
                let expression = Expression::parse(pairs.next().unwrap());
            },
            // Rule::call => {

            // },
            // Rule::if_statement => {

            // },
            Rule::return_statement => {
                unimplemented!();
            },
            _ => panic!("Unexpected {:?}", pair)
        }
    }
}

struct Function {
    name: String,
    args: BTreeSet<String>,
    locals: BTreeSet<String>,
    body: Vec<Statement>,
}

impl Function {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Function {
        assert_eq!(Rule::function, pair.as_rule());

        let mut args = BTreeSet::new();
        let mut locals = BTreeSet::new();

        let mut pairs = pair.into_inner();
        let name = pairs.next().unwrap().as_str().to_owned();
        for arg in pairs.next().unwrap().into_inner() {
            args.insert(arg.as_str().to_owned());
        }

        let body = pairs.next().unwrap().into_inner().map(|p| Statement::parse(p)).collect();

        for s in &body {
            match s {
                //Statement::Assign{local, value} => { },//locals.insert(local); },
                _ => unimplemented!(),
            }
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

    let mut program = ProgramParser::parse(Rule::program, &input).unwrap();
    let pairs = program.next().unwrap().into_inner();
    for pair in pairs {
        match pair.as_rule() {
            Rule::function => {
                //ast.push(Print(Box::new(build_ast_from_expr(pair))));
                println!("Found function: {}\n", pair.as_str());
                Function::parse(pair);
            },
            Rule::EOI => { },
            _ => {
                panic!("Unexpected rule: {:?}", pair);
            }
        }
    }

    // let pairs = program.next().unwrap().into_inner();//map(|p| p.as_rule()).collect();
    // for f in functions {
    //     println!("BEGIN SLICE");
    //     println!("{}", f.as_str());
    //     println!("END SLICE");
    //     match f.as_rule() {
    //         Rule::function(f) => {
    //             println!("Found function");
    //         },
    //     }
    //     println!("BEGIN AST");
    //     println!("{:?}", f.as_rule());
    //     println!("END AST");
    // }
    Ok(())
}
