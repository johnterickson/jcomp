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

// #[derive(Debug)]
// enum Arg0 {
//     Constant (u8),
//     Jump (Jump),
//     Reg (Reg),
// }

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

    let lines : Vec<String> = {
        let stdin = io::stdin();
        stdin.lock().lines().map(|l| l.unwrap()).collect()
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
