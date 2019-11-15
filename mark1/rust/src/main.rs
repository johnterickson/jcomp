extern crate strum;
#[macro_use]
extern crate strum_macros;

use std::io::{self, BufRead};
use std::str::FromStr;

#[derive(Debug, EnumString, PartialEq)]
#[strum(serialize_all = "lowercase")]
enum Reg {
    A,
    B,
    C,
    PC,
}

#[derive(Debug, EnumString)]
#[strum(serialize_all = "lowercase")]
enum Jump {
    Jz,
    Jnz,
    Jg,
    Jl,
}

#[derive(Clone, Copy, Debug, EnumString, PartialEq, PartialOrd)]
#[strum(serialize_all = "lowercase")]
enum Macro {
    LoadImm
}

#[derive(Clone, Copy, Debug, EnumString, PartialEq, PartialOrd)]
#[strum(serialize_all = "lowercase")]
enum OpCode {
    Load,
    Store,
    And,
    Or,
    Add,
    Not,
    Copy,
    Xor,
    LoadLo,
    LoadLo9,
    LoadLoA,
    LoadLoB,
    LoadHi,
    LoadHiD,
    LoadHiE,
    LoadHiF
}

fn main() -> Result<(), std::io::Error> {
    println!("v2.0 raw");

    let lines = {
        let mut lines = Vec::new();
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;
            let mut tokens = line.split_whitespace();
            let first_token = tokens.next();
            match first_token {
                Some("#") | None => { },
                Some("jnz") => {
                    lines.push(format!("copy jnz pc"));
                }
                Some("loadimm") => {
                    let constant = tokens.next().expect("loadimm needs a constant.");
                    let constant = u8::from_str_radix(constant, 16).expect("loadimm constant must be hex");
                    let reg_str = tokens.next().expect("loadimm needs a register.");
                    let reg = Reg::from_str(reg_str).expect("invalid register");
                    assert_ne!(reg, Reg::PC);
                    lines.push(format!("loadlo {:x} {}", constant & 0xF, reg_str));
                    lines.push(format!("loadhi {:x} {}", constant >> 4, reg_str));
                },
                _ => { lines.push(line); }
            }
        }
        lines
    };

    for line in lines {

        println!("# {}", line);

        let tokens : Vec<&str> = line.split_whitespace().collect();
        assert!(tokens.len() == 3);

        let op = OpCode::from_str(tokens[0]).expect("invalid opcode");
        let arg0 = tokens[1];
        let arg1 = Reg::from_str(tokens[2]).expect("invalid register");

        let mut instruction : u8 = (op as u8) << 4;
        match op {
            op if OpCode::Load <= op && op <= OpCode::Xor => {
                if op == OpCode::Copy && arg1 == Reg::PC {
                    let jmp = Jump::from_str(arg0).expect("invalid jump");
                    println!("# {:?} {:?} {:?}", op, jmp, arg1);
                    instruction |= arg1 as u8;
                    instruction |= (jmp as u8) << 2;
                } else {
                    let reg0 = Reg::from_str(arg0).expect("invalid register");
                    println!("# {:?} {:?} {:?}", op, reg0, arg1);
                    instruction |= arg1 as u8;
                    instruction |= (reg0 as u8) << 2;
                }
            }
            op if OpCode::LoadLo <= op && op <= OpCode::LoadHiF => {
                let c = u8::from_str_radix(arg0, 16).expect("invalid constant");
                println!("# {:?} {:?} {:?}", op, c, arg1);
                instruction |= arg1 as u8;
                instruction |= c << 2;
            }
            _ => { unreachable!(); }
        }
        
        println!("{:02x}", instruction);
    }
    println!();
    Ok(())
}
