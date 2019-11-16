extern crate strum;
#[macro_use]
extern crate strum_macros;

use std::io::{self, BufRead};
use std::str::FromStr;

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, EnumString, PartialEq)]
#[strum(serialize_all = "lowercase")]
enum Reg {
    ACC,
    B,
    C,
    D,
    E,
    F,
    SP,
    PC,
    UNKNOWN
}

#[derive(Debug)]
enum Target {
    Label(String),
    Constant(u8),
}

impl Target {
    fn parse(s: &str) -> Target {
        match s.chars().next() {
            Some(':') => Target::Label(s.to_owned()),
            Some(_) => Target::Constant(u8::from_str_radix(s, 16).expect("invalid hex")),
            None => panic!("argument needed.")
        }
    }
}

#[derive(Debug)]
enum Instruction {
    LoadReg(Reg),
    StoreReg(Reg),
    Xor(Reg),
    And(Reg),
    Or(Reg),
    Add(Reg),
    Not(Reg),
    LoadMem(Reg),
    StoreMem(Reg),
    LoadLo(Target),
    LoadHi(Target),
    Jmp(Target),
    Jz(Target),
    Jnz(Target),
}

impl Instruction {
    fn encode(&self, pc: u8, labels: &BTreeMap<&String,u8>) -> u8 {
        match self {
            Instruction::LoadReg(r) => 0x00 | *r as u8,
            Instruction::StoreReg(r) => 0x08 | *r as u8,
            Instruction::Xor(r) => 0x40 | *r as u8,
            Instruction::And(r) => 0x48 | *r as u8,
            Instruction::Or(r) => 0x50 | *r as u8,
            Instruction::Add(r) => 0x58 | *r as u8,
            Instruction::Not(r) => 0x60 | *r as u8,
            Instruction::LoadMem(r) => 0x70 | *r as u8,
            Instruction::StoreMem(r) => 0x78 | *r as u8,
            Instruction::LoadLo(t) => 0x80 | match t {
                Target::Constant(c) => *c,
                Target::Label(l) => labels[l] & 0xf
            },
            Instruction::LoadHi(t) => 0x90 | match t {
                Target::Constant(c) => *c,
                Target::Label(l) => (labels[l] >> 4) & 0xf
            },
            Instruction::Jmp(t) => 0xa0 | match t {
                Target::Constant(c) => *c,
                Target::Label(l) => {
                    let mut offset : i16 = labels[l] as i16;
                    offset -= pc as i16;
                    (offset & 0x1f) as u8
                }
            },
            Instruction::Jz(t) => 0xa0 | match t {
                Target::Constant(c) => *c,
                Target::Label(l) => {
                    let mut offset : i16 = labels[l] as i16;
                    offset -= pc as i16;
                    (offset & 0x1f) as u8
                }
            },
            Instruction::Jnz(t) => 0xa0 | match t {
                Target::Constant(c) => *c,
                Target::Label(l) => {
                    let mut offset : i16 = labels[l] as i16;
                    offset -= pc as i16;
                    (offset & 0x1f) as u8
                }
            },
            _ => {unimplemented!();}
        }
    }

    fn parse(line: &str) -> Vec<Instruction> {
        let mut instructions = Vec::new();

        let tokens : Vec<&str> = line.split_whitespace().collect();

        match tokens[0].to_lowercase().as_ref() {
            "loadreg" => instructions.push(Instruction::LoadReg(Reg::from_str(tokens[1]).expect("invalid register"))),
            "storereg" => instructions.push(Instruction::StoreReg(Reg::from_str(tokens[1]).expect("invalid register"))),
            "xor" => instructions.push(Instruction::Xor(Reg::from_str(tokens[1]).expect("invalid register"))),
            "and" => instructions.push(Instruction::And(Reg::from_str(tokens[1]).expect("invalid register"))),
            "or" => instructions.push(Instruction::Or(Reg::from_str(tokens[1]).expect("invalid register"))),
            "add" => instructions.push(Instruction::Add(Reg::from_str(tokens[1]).expect("invalid register"))),
            "not" => instructions.push(Instruction::Not(Reg::from_str(tokens[1]).expect("invalid register"))),
            "loadmem" => instructions.push(Instruction::LoadMem(Reg::from_str(tokens[1]).expect("invalid register"))),
            "storemem" => instructions.push(Instruction::StoreMem(Reg::from_str(tokens[1]).expect("invalid register"))),
            "loadlo" => instructions.push(Instruction::LoadLo(Target::parse(tokens[1]))),
            "loadhi" => instructions.push(Instruction::LoadHi(Target::parse(tokens[1]))),
            "jmp" => instructions.push(Instruction::Jmp(Target::parse(tokens[1]))),
            "jz" => instructions.push(Instruction::Jz(Target::parse(tokens[1]))),
            "jnz" => instructions.push(Instruction::Jnz(Target::parse(tokens[1]))),
            "push" => {
                let reg = Reg::from_str(tokens[1]).expect("invalid register");
                instructions.push(Instruction::LoadLo(Target::Constant(0xF)));
                instructions.push(Instruction::Add(Reg::SP));
                instructions.push(Instruction::StoreReg(Reg::SP));
                instructions.push(Instruction::LoadReg(reg));
                instructions.push(Instruction::StoreMem(Reg::SP));
            },
            "pop" => {
                let reg = Reg::from_str(tokens[1]).expect("invalid register");
                instructions.push(Instruction::LoadMem(Reg::SP));
                instructions.push(Instruction::StoreReg(reg));
                instructions.push(Instruction::LoadLo(Target::Constant(0x1)));
                instructions.push(Instruction::Add(Reg::SP));
                instructions.push(Instruction::StoreReg(Reg::SP));
            },
            "call" => {
                instructions.push(Instruction::LoadLo(Target::Constant(0xF)));
                instructions.push(Instruction::Add(Reg::SP));
                instructions.push(Instruction::StoreReg(Reg::SP));
                instructions.push(Instruction::LoadLo(Target::Constant(0x5))); //TODO is this wrong?
                instructions.push(Instruction::Add(Reg::PC));
                instructions.push(Instruction::StoreMem(Reg::SP));
                instructions.push(Instruction::LoadLo(Target::Label(tokens[1].to_owned())));
                instructions.push(Instruction::LoadHi(Target::Label(tokens[1].to_owned())));
                instructions.push(Instruction::StoreReg(Reg::PC));
            },
            "ret" => {
                instructions.push(Instruction::LoadMem(Reg::SP));
                instructions.push(Instruction::StoreReg(Reg::PC));
            }
            _ => unimplemented!(),
        }
        instructions
    }
}

#[derive(Debug)]
enum Line {
    Label(String),
    Comment(String),
    Instruction(Instruction),
}

fn main() -> Result<(), std::io::Error> {

    let lines = {
        let mut lines = Vec::new();
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;

            match line.chars().next() {
                Some('#') | None => { 
                    lines.push(Line::Comment(line));
                },
                Some(':') => {
                    lines.push(Line::Label(line));
                }
                _ => {
                    for instruction in Instruction::parse(&line) {
                        lines.push(Line::Instruction(instruction));
                    }
                },
            }
        }
        lines
    };

    let labels = {
        let mut labels = BTreeMap::new();
        let mut address : u8= 0;
        for line in &lines {
            match line {
                Line::Instruction(_) => { address += 1; },
                Line::Label(l) => { labels.insert(l, address); }
                Line::Comment(_) => {}
            }
        }
        labels
    };

    println!("v2.0 raw");

    for l in &labels {
        println!("# {:?}", l);
    }

    let mut pc = 0;
    for l in &lines {
        match l {
            Line::Instruction(i) => {
                let instruction = i.encode(pc, &labels);
                println!("{:02x} # {:?}", instruction, i);
                pc += 1;
            },
            Line::Label(l) => { 
                println!("# {:?}", l);
            }
            Line::Comment(c) => { 
                println!("# {}", c);
            }
        }
    }

        
    // }
    println!();
    Ok(())
}
