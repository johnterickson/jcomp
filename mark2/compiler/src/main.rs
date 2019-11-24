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
                let mut n = 0;
                let mut digits = pair.into_inner();
                while let Some(digit) = digits.next() {
                    let digit = i32::from_str(digit.as_str()).expect("Couldn't parse integer.");
                    n *= 10;
                    n += digit;
                }
                Expression::Number(n)
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

    // output is in b; must preserve other regs
    fn emit(&self, stack: &BTreeMap<String,isize>, additional_offset: isize) -> () {
        match self {
            Expression::Number(n) => {
                println!("loadlo {:02x}", n & 0xF);
                println!("loadhi {:02x}", (n>>4) & 0xF);
                println!("storereg b");
            },
            Expression::Ident(n) => {
                println!("loadlo {:02x}", stack[n] + additional_offset);
                println!("add sp");
                println!("loadmem acc");
                println!("storereg b");
            },
            Expression::Operation(op, left, right) => {
                println!("push c");
                left.emit(&stack, additional_offset+1);
                println!("push b"); //store left on the stack
                right.emit(&stack, additional_offset+2);
                // left on top of stack; right in b
                match op {
                    Operator::Add => {
                        println!("pop c"); //left in c; right in b
                        println!("loadreg b"); // left in acc; right in b
                        println!("add b"); // add right to left
                        println!("storereg b");
                    },
                    _ => unimplemented!()
                }
                println!("pop c"); //restore
            }
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

        for arg in pairs.next().unwrap().into_inner() {
            let arg = arg.as_str();
            args.push(arg.to_owned());
        }

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

        Function { name, args, locals, body }
    }

    /*

    stack:

    SP ->   local 3
            local 2
            local 1
            saved c
            saved b
            return address
            arg 2
            arg 1
    */

    fn emit(&self) -> () {
        println!("# Function: {}", &self.name);
        println!(":{}", &self.name);

        let mut stack = BTreeMap::new();
        let stack_size = 0
            + 1 // result
            + self.args.len() 
            + 1 // return address
            + 2 // save b and c
            + self.locals.len();
        let mut offset = (stack_size - 1) as isize;

        let RESULT = "RESULT";
        let EPILOGUE = "EPILOGUE";

        println!("# sp+{} -> {}", offset, RESULT);
        stack.insert(RESULT.to_owned(), offset);
        offset -= 1;

        for arg in &self.args {
            println!("# sp+{} -> {}", offset, arg);
            stack.insert(arg.clone(), offset);
            offset -= 1;
        }

        println!("# sp+{} -> {}", offset, "RETURN_ADDRESS");
        stack.insert("RETURN_ADDRESS".to_owned(), offset);
        offset -= 1;

        println!("# sp+{} -> {}", offset, "saved b");
        offset -= 1;
        println!("# sp+{} -> {}", offset, "saved c");
        offset -= 1;

        for l in &self.locals {
            println!("# sp+{} -> {}", offset, l);
            stack.insert(l.clone(), offset);
            offset -= 1;
        }

        assert_eq!(-1, offset);

        println!("# save registers");
        println!("push b");
        println!("push c");

        println!("# create stack space");
        println!("loadlo {:2x}", ((self.locals.len() as i32) * -1) & 0xF );
        println!("add sp");
        println!("storereg sp");

        for stmt in self.body.iter() {
            println!("# {:?}", stmt);
            match stmt {
                Statement::Assign{local, value} => {

                    println!("loadlo {:02x}", stack[local]);
                    println!("add sp");
                    println!("storereg c");

                    value.emit(&stack, 0);
                    println!("loadreg b");
                    println!("storemem c");
                    
                },
                Statement::Return{ local } => {
                    println!("loadlo {:02x}", stack[RESULT]);
                    println!("add sp");
                    println!("storereg b");
                    println!("loadlo {:02x}", stack[local]);
                    println!("add sp");
                    println!("storereg c");
                    println!("loadmem c");
                    println!("storemem b");
                    println!("jmp :{}__{}", &self.name, EPILOGUE);
                },
                Statement::Call{ local, function:_, parameters:_ } => { 
                },
                Statement::If{ predicate:_, when_true:_ } => {
                    unimplemented!();
                },
            }
        }
         
        println!(":{}__{}", &self.name, EPILOGUE);
        println!("loadlo {}", self.locals.len());
        println!("add sp");
        println!("storereg sp");
        println!("pop c");
        println!("pop b");
        println!("ret");
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

    println!("# set stack to 0xff");
    println!("loadlo f");
    println!("storereg sp");

    println!("# call main");
    println!("loadlo f");
    println!("add sp");
    println!("storereg sp");
    println!("call :main");
    println!("pop b");
    println!("halt");
    
    for f in &functions {
        f.1.emit();
    }

    Ok(())
}
