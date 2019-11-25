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
    Equals,
    NotEquals
}

impl Operator {
    fn parse(pair: pest::iterators::Pair<Rule>) -> Operator {
        match pair.as_str() {
            "+" => Operator::Add,
            "-" => Operator::Subtract,
            "*" => Operator::Multiply,
            "||" => Operator::Or,
            "==" => Operator::Equals,
            "!=" => Operator::NotEquals,
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
                let mut label = String::new();
                let mut chars = pair.into_inner();
                while let Some(c) = chars.next() {
                    label += c.as_str();
                }
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
                let base_offset = stack.get(n).expect(&format!("could not find {}", n));
                println!("loadlo {:02x}", base_offset + additional_offset);
                println!("add sp");
                println!("loadmem acc");
                println!("storereg b");
            },
            Expression::Operation(op, left, right) => {
                left.emit(&stack, additional_offset);
                println!("push b"); //store left on the stack
                right.emit(&stack, additional_offset+1);
                // left on top of stack; right in b
                println!("pop c"); //left in c; right in b

                match op {
                    Operator::Add => {
                        println!("loadreg b"); // left in c; right in acc
                        println!("add c"); // add left to right
                        println!("storereg b");
                    },
                    Operator::Multiply => {
                        println!("loadreg b"); // left in c; right in acc
                        println!("mul c");
                        println!("storereg b");
                    },
                    Operator::Subtract => {
                        println!("not b"); // b <- ~right
                        println!("storereg b");
                        println!("loadlo 1");
                        println!("add b"); // acc <- (~right) + 1 aka -1*right
                        println!("add c"); // add left to right
                        println!("storereg b");
                    },
                    Operator::Equals => {
                        println!("loadreg b");
                        println!("xor c"); //  left ^ right == 0 --> left == right
                        println!("storereg b");
                    },
                    _ => unimplemented!()
                }
            }
        }
    }
}

const RESULT : &'static str = "RESULT";
const EPILOGUE : &'static str = "EPILOGUE";

#[derive(Debug)]
enum Statement {
    Assign {local: String, value: Expression},
    Call { local: String, function: String, parameters: Vec<Expression> },
    If {predicate: Expression, when_true: Vec<Statement> },
    Return { value: Expression},
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
                let expr = pair.into_inner().next().unwrap();
                Statement::Return { value: Expression::parse(expr) }
            },
            _ => panic!("Unexpected {:?}", pair)
        }
    }

    fn emit(&self, function_name: &str, scope: &str, stack: &BTreeMap<String,isize>, additional_offset: isize) -> () {
        println!("# {:?}", self);
        match self {
            Statement::Assign{local, value} => {
                value.emit(&stack, 0);
                println!("loadlo {:02x}", stack[local] + additional_offset);
                println!("add sp");
                println!("storereg c");
                println!("loadreg b");
                println!("storemem c");
                
            },
            Statement::Return{ value } => {
                value.emit(&stack, 0);
                println!("loadlo {:02x}", stack[RESULT] + additional_offset);
                println!("add sp");
                println!("storereg c");
                println!("loadreg b");
                println!("storemem c");
                if additional_offset != 0 {
                    println!("loadlo {:02x}", additional_offset);
                    println!("add sp");
                    println!("storereg sp");
                }
                println!("loadlo :{}__{}", function_name, EPILOGUE);
                println!("loadhi :{}__{}", function_name, EPILOGUE);
                println!("storereg pc");
            },
            Statement::Call{ local, function, parameters} => { 

                let mut stack_offset = 0;
                println!("dec sp"); // save space for result
                stack_offset += 1;

                for p in parameters {
                    p.emit(&stack, stack_offset);
                    println!("push b");
                    stack_offset += 1;
                }

                println!("call :{}", function);

                // discard paramters
                println!("loadlo {:02x}", parameters.len());
                println!("add sp");
                println!("storereg sp"); 

                // pop result into b
                println!("pop b");

                // stack is now back to normal

                // c = &local
                println!("loadlo {:02x}", stack[local] + additional_offset);
                println!("add sp");
                println!("storereg c"); 

                println!("loadreg b");
                println!("storemem c");

            },
            Statement::If{ predicate, when_true} => {
                let IF_SKIP = "IF_SKIP";
                
                predicate.emit(&stack, 0); // result in b
                println!("loadreg b");

                let jump_label = format!("{}_{}_{}", function_name, scope, IF_SKIP);

                // WEIRD: interpret 0 as true

                println!("jnz :{}", &jump_label);

                let mut count = 0;
                for s in when_true {
                    let scope = format!("{}_stmt{}", scope, count);
                    s.emit(function_name, &scope, &stack, 0);
                    count += 1;
                }
                
                println!(":{}", &jump_label);
            },
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
                Statement::Return{ value:_ } => {},
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
            return address
            arg 2
            arg 1
            RESULT
    */

    fn emit(&self) -> () {
        println!("# Function: {}", &self.name);
        println!(":{}", &self.name);

        let mut stack = BTreeMap::new();
        let stack_size = 0
            + 1 // result
            + self.args.len() 
            + 1 // return address
            + self.locals.len();
        let mut offset = (stack_size - 1) as isize;

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

        // println!("# sp+{} -> {}", offset, "saved c");
        // offset -= 1;

        for l in &self.locals {
            println!("# sp+{} -> {}", offset, l);
            stack.insert(l.clone(), offset);
            offset -= 1;
        }

        assert_eq!(-1, offset);

        // println!("# save registers");
        // println!("push c");

        println!("# create stack space");
        println!("loadlo {:2x}", ((self.locals.len() as i32) * -1) & 0xF );
        println!("add sp");
        println!("storereg sp");

        let mut count = 0;
        for stmt in self.body.iter() {
            let scope = format!("_function{}_", count);
            stmt.emit(&self.name, &scope, &stack, 0);
            count += 1;
        }
         
        println!(":{}__{}", &self.name, EPILOGUE);
        println!("loadlo {}", self.locals.len());
        println!("add sp");
        println!("storereg sp");
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
