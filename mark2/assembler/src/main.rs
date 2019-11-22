// extern crate strum;

use std::io::{self, BufRead};


use std::collections::BTreeMap;
use common::*;

trait Resolver {
    fn resolve(&self, pc: u8, labels: &BTreeMap<&String,u8>) -> Instruction;
}

impl Resolver for Instruction {

    fn resolve(&self, pc: u8, labels: &BTreeMap<&String,u8>) -> Instruction {
        match self {
            Instruction::LoadLo(t) => match t {
                Target::Constant(_) => self.clone(),
                Target::Label(l) => Instruction::LoadLo(Target::Constant(labels[l] & 0xf)),
            },
            Instruction::LoadHi(t) => match t {
                Target::Constant(_) => self.clone(),
                Target::Label(l) => Instruction::LoadHi(Target::Constant((labels[l] >> 4) & 0xf)),
            },
            Instruction::Jmp(t) => match t {
                Target::Constant(_) => self.clone(),
                Target::Label(l) => {
                    let mut offset : i16 = labels[l] as i16;
                    offset -= pc as i16;
                    assert!(offset <= 15);
                    assert!(offset >= -16);
                    Instruction::Jmp(Target::Constant((offset & 0x1f) as u8))
                }
            },
            Instruction::Jz(t) => match t {
                Target::Constant(_) => self.clone(),
                Target::Label(l) => {
                    let mut offset : i16 = labels[l] as i16;
                    offset -= pc as i16;
                    assert!(offset <= 15);
                    assert!(offset >= -16);
                    Instruction::Jz(Target::Constant((offset & 0x1f) as u8))
                }
            },
            Instruction::Jnz(t) => match t {
                Target::Constant(_) => self.clone(),
                Target::Label(l) => {
                    let mut offset : i16 = labels[l] as i16;
                    offset -= pc as i16;
                    assert!(offset <= 15);
                    assert!(offset >= -16);
                    Instruction::Jnz(Target::Constant((offset & 0x1f) as u8))
                }
            },
            _ => self.clone()
        }
    }
}

fn main() -> Result<(), std::io::Error> {

    let lines = {
        let mut lines = Vec::new();
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;
            lines.push(Line::parse(line));
        }
        lines
    };

    let labels = {
        let mut labels = BTreeMap::new();
        let mut address : u8 = 0;
        for line in &lines {
            match line {
                Line::Instruction(_) => { address += 1; },
                Line::Label(l) => { labels.insert(l, address); }
                Line::Comment(_) => {},
                Line::Macro(_, instructions) => { address += instructions.len() as u8; }
            }
        }
        labels
    };

    println!("v2.0 raw");

    for l in &labels {
        println!("# {:?}", l);
    }

    let rom = {
        let mut instructions = Vec::new();
        let mut pc = 0;

        let mut resolve_instruction = |i: &Instruction| {
            let resolved = i.resolve(pc, &labels);
            print!("{:02x} # @{:02x} {:?}", resolved.encode(), pc, i);
            if *i != resolved {
                print!(" {:?}", resolved);
            }
            println!();
            instructions.push(resolved);
            pc += 1;
        };

        for l in &lines {
            match l {
                Line::Instruction(i) => {
                    resolve_instruction(i);
                },
                Line::Macro(line, instructions) => {
                    println!("# begin resolving macro: '{}'", &line);
                    for i in instructions {
                        resolve_instruction(i);
                    }
                    println!("# end resolving macro: '{}'", &line);
                },
                Line::Label(l) => { 
                    println!("# {:?}", l);
                },
                Line::Comment(c) => { 
                    println!("# {}", c);
                },
            }
        }
        instructions
    };
    
    simulate(&rom, 10000);

    Ok(())
}
