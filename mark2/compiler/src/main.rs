extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

use std::io;
use std::io::ErrorKind;
use std::io::Read;
use std::collections::{BTreeMap,BTreeSet};
use std::str::FromStr;

use common::*;

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

struct FunctionContext {
    pub regs_touched: BTreeSet<Reg>,
    pub stack: BTreeMap<String, LocalStorage>,
    pub lines: Vec<Line>,
    pub additional_offset: isize,
    pub block_counter: usize,
}

impl FunctionContext {
    fn add_inst(&mut self, i: Instruction) {
        //println!("{:?}",&i);
        self.lines.push(Line::Instruction(i));
    }

    fn add_macro(&mut self, s: String) {
        let line = Line::parse(s);
        self.lines.push(line);
    }

    fn find_local(&mut self, local: &str) -> LocalStorage {
        let local = self.stack
            .get(local)
            .expect(&format!("could not find {}", local));
        match local {
            LocalStorage::Stack(offset) => {
                LocalStorage::Stack(offset + self.additional_offset)
            },
            LocalStorage::Register(r) => {
                self.regs_touched.insert(*r);
                LocalStorage::Register(*r)
            }
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

    fn is_tail(&self) -> bool {
        match self {
            Expression::Ident(_) => true,
            Expression::Number(_) => true,
            Expression::Operation(_,_,_) => false,
        }
    }

    // output is in target_reg; must preserve other regs
    fn emit(&self, ctxt: &mut FunctionContext, target_reg: Reg) -> () {
        ctxt.lines.push(Line::Comment(format!("Evaluating expression: {:?}", &self)));
        match self {
            Expression::Number(n) => {
                ctxt.add_inst(Instruction::LoadLo(Target::Constant((n & 0xF) as u8)));

                // todo: skip this when we can
                ctxt.add_inst(Instruction::LoadHi(Target::Constant(((n>>4) & 0xF) as u8)));
            },
            Expression::Ident(n) => {
                let local = ctxt.find_local(n);
                match local {
                    LocalStorage::Register(r) => {
                        ctxt.add_inst(Instruction::LoadReg(r));
                    },
                    LocalStorage::Stack(offset) => {
                        ctxt.add_inst(Instruction::LoadLo(Target::Constant(offset as u8)));
                        ctxt.add_inst(Instruction::Add(Reg::SP));
                        ctxt.add_inst(Instruction::LoadMem(Reg::ACC));
                    }
                }
            },
            Expression::Operation(op, left, right) => {
                if left.is_tail() && right.is_tail() {
                    left.emit(ctxt, Reg::C);
                    right.emit(ctxt, Reg::ACC)
                }
                else 
                {
                    left.emit(ctxt, Reg::B); // left in B
                    ctxt.add_macro(format!("push b")); //store left on the stack
                    ctxt.additional_offset += 1;

                    right.emit(ctxt, Reg::B); // left on top of stack; right in b
                    ctxt.add_macro(format!("pop c")); //left in c; right in b
                    ctxt.additional_offset -= 1;
                    ctxt.add_inst(Instruction::LoadReg(Reg::B)); 
                }

                // left in c; right in acc

                match op {
                    Operator::Add => {
                        ctxt.add_inst(Instruction::Add(Reg::C)); // add left to right
                    },
                    Operator::Multiply => {
                        ctxt.add_inst(Instruction::Mul(Reg::C));
                    },
                    Operator::Subtract => {
                        ctxt.add_inst(Instruction::Not(Reg::ACC)); // b <- ~right
                        ctxt.add_inst(Instruction::StoreReg(Reg::B));
                        ctxt.add_inst(Instruction::LoadLo(Target::Constant(1)));
                        ctxt.add_inst(Instruction::Add(Reg::B)); // acc <- (~right) + 1 aka -1*right
                        ctxt.add_inst(Instruction::Add(Reg::C)); // add left to right
                    },
                    Operator::Equals => {
                        ctxt.add_inst(Instruction::Xor(Reg::C)); //  left ^ right == 0 --> left == right
                    },
                    _ => unimplemented!()
                }
            }
        }
        if target_reg != Reg::ACC {
            ctxt.add_inst(Instruction::StoreReg(target_reg));
        }
        ctxt.lines.push(Line::Comment(format!("Evaluated expression: {:?}", &self)));
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
                let local = pairs.next().unwrap().as_str().trim().to_owned();
                let value = Expression::parse(pairs.next().unwrap());
                Statement::Assign { local, value }
            },
            Rule::call => {
                let mut pairs = pair.into_inner();
                let local = pairs.next().unwrap().as_str().trim().to_owned();
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

    fn emit(&self, ctxt: &mut FunctionContext, function_name: &str) -> () {
        ctxt.lines.push(Line::Comment(format!("Begin statement {:?}", self)));
        match self {
            Statement::Assign{local, value} => {
                
                let local = ctxt.find_local(local);
                match local {
                    LocalStorage::Register(r) => {
                        value.emit(ctxt, r);
                    }
                    LocalStorage::Stack(offset) => {
                        value.emit(ctxt, Reg::B);
                        ctxt.add_inst(Instruction::LoadLo(Target::Constant((offset & 0xf) as u8)));
                        ctxt.add_inst(Instruction::Add(Reg::SP));
                        ctxt.add_inst(Instruction::StoreReg(Reg::C));
                        ctxt.add_inst(Instruction::LoadReg(Reg::B));
                        ctxt.add_inst(Instruction::StoreMem(Reg::C));
                    }
                }
            },
            Statement::Return{ value } => {
                value.emit(ctxt, Reg::B);

                let result_offset = match ctxt.find_local(RESULT) {
                    LocalStorage::Register(_) => unimplemented!(),
                    LocalStorage::Stack(offset) => offset,
                };

                ctxt.add_inst(Instruction::LoadLo(Target::Constant(
                    (result_offset & 0xf) as u8
                )));
                ctxt.add_inst(Instruction::Add(Reg::SP));
                ctxt.add_inst(Instruction::StoreReg(Reg::C));
                ctxt.add_inst(Instruction::LoadReg(Reg::B));
                ctxt.add_inst(Instruction::StoreMem(Reg::C));
                if ctxt.additional_offset != 0 {
                    ctxt.add_inst(Instruction::LoadLo(Target::Constant(
                        (ctxt.additional_offset & 0xf) as u8
                    )));
                    ctxt.add_inst(Instruction::Add(Reg::SP));
                    ctxt.add_inst(Instruction::StoreReg(Reg::SP));
                }
                ctxt.add_inst(Instruction::LoadLo(Target::Label(
                    format!(":{}__{}", function_name, EPILOGUE)
                )));
                ctxt.add_inst(Instruction::LoadHi(Target::Label(
                    format!(":{}__{}", function_name, EPILOGUE)
                )));
                ctxt.add_inst(Instruction::StoreReg(Reg::PC));
            },
            Statement::Call{ local, function, parameters} => { 

                assert_eq!(ctxt.additional_offset, 0);
                ctxt.add_macro(format!("dec sp")); // save space for result
                ctxt.additional_offset += 1;

                let regs_to_save : Vec<Reg> = ctxt.regs_touched.iter().cloned().collect();

                for r in &regs_to_save {
                    ctxt.add_macro(format!("push {}", r));
                    ctxt.additional_offset += 1;
                }

                for p in parameters {
                    p.emit(ctxt, Reg::B);
                    ctxt.add_macro(format!("push b"));
                    ctxt.additional_offset += 1;
                }

                ctxt.add_macro(format!("call :{}", function));

                // discard paramters
                ctxt.add_inst(Instruction::LoadLo(Target::Constant(
                    (parameters.len() & 0xf) as u8
                )));
                ctxt.add_inst(Instruction::Add(Reg::SP));
                ctxt.add_inst(Instruction::StoreReg(Reg::SP)); 

                ctxt.additional_offset -= parameters.len() as isize;

                for r in regs_to_save.iter().rev() {
                    ctxt.add_macro(format!("pop {}", r));
                    ctxt.additional_offset -= 1;
                }

                // pop result into b
                ctxt.add_macro(format!("pop b"));
                ctxt.additional_offset -= 1;

                // stack is now back to normal

                let local = ctxt.find_local(local);
                match local {
                    LocalStorage::Register(r) => {
                        ctxt.add_inst(Instruction::LoadReg(Reg::B));
                        ctxt.add_inst(Instruction::StoreReg(r));

                    },
                    LocalStorage::Stack(offset) => {
                        ctxt.add_inst(Instruction::LoadLo(Target::Constant(
                            (offset & 0xf) as u8
                        )));
                        ctxt.add_inst(Instruction::Add(Reg::SP));
                        ctxt.add_inst(Instruction::StoreReg(Reg::C));
        
                        ctxt.add_inst(Instruction::LoadReg(Reg::B));
                        ctxt.add_inst(Instruction::StoreMem(Reg::C));
                    }
                }
            },
            Statement::If{predicate, when_true} => {
                let if_skip = "IF_SKIP";
                predicate.emit(ctxt, Reg::ACC); // result in ACC

                let jump_label = format!("{}_{}_{}", function_name, if_skip, ctxt.block_counter);

                ctxt.block_counter += 1;

                // WEIRD: interpret 0 as true
                ctxt.add_inst(Instruction::Jnz(Target::Label(format!(":{}", &jump_label))));

                // let mut count = 0;
                for s in when_true {
                    // let scope = format!("{}_stmt{}", scope, count);
                    s.emit(ctxt, function_name);
                    // count += 1;
                }
                
                ctxt.lines.push(Line::Label(format!(":{}", &jump_label)));
            },
        }
        ctxt.lines.push(Line::Comment(format!("Done  statement {:?}", self)));
    }
}

#[derive(Clone, Copy, Debug)]
enum LocalStorage {
    Register(Reg),
    Stack(isize),
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

        let mut pairs = pair.into_inner();

        let name = pairs.next().unwrap().as_str().to_owned();

        for arg in pairs.next().unwrap().into_inner() {
            let arg = arg.as_str();
            args.push(arg.to_owned());
        }

        let body : Vec<Statement> = pairs.next().unwrap().into_inner().map(|p| Statement::parse(p)).collect();

        // find locals
        let mut locals = BTreeSet::new();
        for s in body.iter() {
            match s {
                Statement::Assign{local, value:_} => { 
                    if !args.contains(local) {
                        locals.insert(local.clone()); 
                    }
                },
                Statement::Return{ value:_ } => {},
                Statement::Call{ local, function:_, parameters:_ } => { 
                    if !args.contains(local) {
                        locals.insert(local.clone()); 
                    }
                },
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
            return address
            arg 2
            arg 1
            RESULT
    */

    fn emit(&self) -> FunctionContext {
        let mut ctxt = FunctionContext {
            stack: BTreeMap::new(),
            lines: Vec::new(),
            additional_offset: 0,
            regs_touched: BTreeSet::new(),
            block_counter: 0,
        };
        ctxt.lines.push(Line::Comment(format!("# Function: {}", &self.name)));
        ctxt.lines.push(Line::Label(format!(":{}", &self.name)));

        let register_local_count = std::cmp::min(2, self.locals.len());
        let stack_local_count = self.locals.len() - register_local_count;

        let stack_size = 0
            + 1 // result
            + self.args.len()
            + 1 // return address
            + stack_local_count;
        let mut offset = (stack_size - 1) as isize;

        ctxt.lines.push(Line::Comment(format!("# sp+{} -> {}", offset, RESULT)));
        ctxt.stack.insert(RESULT.to_owned(), LocalStorage::Stack(offset));
        offset -= 1;

        for arg in &self.args {
            ctxt.lines.push(Line::Comment(format!("# sp+{} -> {}", offset, arg)));
            ctxt.stack.insert(arg.clone(), LocalStorage::Stack(offset));
            offset -= 1;
        }

        ctxt.lines.push(Line::Comment(format!("# sp+{} -> {}", offset, "RETURN_ADDRESS")));
        ctxt.stack.insert("RETURN_ADDRESS".to_owned(), LocalStorage::Stack(offset));
        offset -= 1;

        // offset -= register_local_count as isize;

        for (count, l) in self.locals.iter().enumerate() {
            let storage = match count {
                count if count < register_local_count => {
                    let reg = if count == 0 { Reg::D } else { Reg::E };
                    LocalStorage::Register(reg)
                },
                _ => {
                    let s = LocalStorage::Stack(offset);
                    offset -= 1;
                    s
                }
            };

            ctxt.lines.push(Line::Comment(format!("# {:?} -> {}", storage, l)));
            ctxt.stack.insert(l.clone(), storage);
        }

        assert_eq!(-1, offset);

        // assert_eq!(ctxt.regs_used.len(), register_local_count);
        // if register_local_count > 0 {
        //     ctxt.lines.push(Line::Comment(format!("save regs: {:?}", ctxt.regs_used)));
        //     let regs : Vec<Reg> = ctxt.regs_used.iter().cloned().collect();
        //     for r in regs {
        //         ctxt.add_macro(format!("push {}", r));
        //     }
        // }

        if stack_local_count > 0 {
            ctxt.lines.push(Line::Comment("create stack space".to_owned()));
            ctxt.add_inst(Instruction::LoadLo(
                Target::Constant((((stack_local_count as i32) * -1) & 0xF) as u8)
            ));
            ctxt.add_inst(Instruction::Add(Reg::SP));
            ctxt.add_inst(Instruction::StoreReg(Reg::SP));
        }

        // let mut count = 0;
        for stmt in self.body.iter() {
            // let scope = format!("_function{}_", count);
            stmt.emit(&mut ctxt, &self.name);
            // count += 1;
        }
         
        ctxt.lines.push(Line::Label(format!(":{}__{}", &self.name, EPILOGUE)));
        if stack_local_count > 0 {
            ctxt.add_inst(Instruction::LoadLo(Target::Constant(
                (stack_local_count & 0xf) as u8
            )));
            ctxt.add_inst(Instruction::Add(Reg::SP));
            ctxt.add_inst(Instruction::StoreReg(Reg::SP));
        }

        // if register_local_count > 0 {
        //     ctxt.lines.push(Line::Comment(format!("save regs: {:?}", ctxt.regs_used)));
        //     let regs : Vec<Reg> = ctxt.regs_used.iter().cloned().rev().collect();
        //     for r in regs {
        //         ctxt.add_macro(format!("pop {}", r));
        //     }
        // }

        ctxt.add_macro(format!("ret"));

        ctxt
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

    let mut program = Vec::new();

    program.push(Line::Comment(format!("set stack to 0xff")));
    program.push(Line::Instruction(Instruction::LoadLo(Target::Constant(0xf))));
    program.push(Line::Instruction(Instruction::StoreReg(Reg::SP)));

    program.push(Line::Comment(format!("call main")));

    program.push(Line::Instruction(Instruction::LoadLo(Target::Constant(0xf))));
    program.push(Line::Instruction(Instruction::Add(Reg::SP)));
    program.push(Line::Instruction(Instruction::StoreReg(Reg::SP)));
    program.push(Line::parse(format!("call :main")));
    program.push(Line::parse(format!("pop b")));
    program.push(Line::parse(format!("halt")));

    for f in &functions {
        program.push(Line::Comment(format!("{:?}", &f.1)));
        let f = f.1.emit();
        for l in f.lines {
            program.push(l);
        }
    }

    let rom = assemble(program);

    simulate(&rom, 10000);

    Ok(())
}
