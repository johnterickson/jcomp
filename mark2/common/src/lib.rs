extern crate strum;
#[macro_use]
extern crate strum_macros;

use std::str::FromStr;

#[derive(Clone, Copy, Debug, EnumString, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum Reg {
    ACC,
    B,
    C,
    D,
    E,
    FLAGS,
    SP,
    PC,
    UNKNOWN
}

#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    Label(String),
    Constant(u8),
}

impl Target {
    pub fn parse(s: &str) -> Target {
        match s.chars().next() {
            Some(':') => Target::Label(s.to_owned()),
            Some(_) => Target::Constant(u8::from_str_radix(s, 16).expect("invalid hex")),
            None => panic!("argument needed.")
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    LoadReg(Reg),
    StoreReg(Reg),
    Xor(Reg),
    And(Reg),
    Or(Reg),
    Add(Reg),
    Not(Reg),
    Mul(Reg),
    LoadMem(Reg),
    StoreMem(Reg),
    LoadLo(Target),
    LoadHi(Target),
    Jmp(Target),
    Jz(Target),
    Jnz(Target),
}

impl Instruction {

    pub fn encode(&self) -> u8 {
        match self {
            Instruction::StoreReg(r) => 0x00 | *r as u8,
            Instruction::StoreMem(r) => 0x08 | *r as u8,
            Instruction::Xor(r) => 0x40 | *r as u8,
            Instruction::And(r) => 0x48 | *r as u8,
            Instruction::Or(r) => 0x50 | *r as u8,
            Instruction::Add(r) => 0x58 | *r as u8,
            Instruction::Not(r) => 0x60 | *r as u8,
            Instruction::Mul(r) => 0x68 | *r as u8,
            Instruction::LoadReg(r) => 0x70 | *r as u8,
            Instruction::LoadMem(r) => 0x78 | *r as u8,
            Instruction::LoadLo(t) => 0x80 | match t {
                Target::Constant(c) => *c,
                _ => unreachable!(),
            },
            Instruction::LoadHi(t) => 0x90 | match t {
                Target::Constant(c) => *c,
                _ => unreachable!(),
            },
            Instruction::Jmp(t) => 0xa0 | match t {
                Target::Constant(c) => *c,
                _ => unreachable!(),
            },
            Instruction::Jz(t) => 0xc0 | match t {
                Target::Constant(c) => *c,
                _ => unreachable!(),
            },
            Instruction::Jnz(t) => 0xe0 | match t {
                Target::Constant(c) => *c,
                _ => unreachable!(),
            },
        }
    }

    pub fn parse(line: &str) -> Instruction {
        let tokens : Vec<&str> = line.split_whitespace().collect();

        match tokens[0].to_lowercase().as_ref() {
            "loadreg" => Instruction::LoadReg(Reg::from_str(tokens[1]).expect("invalid register")),
            "storereg" => Instruction::StoreReg(Reg::from_str(tokens[1]).expect("invalid register")),
            "xor" => Instruction::Xor(Reg::from_str(tokens[1]).expect("invalid register")),
            "and" => Instruction::And(Reg::from_str(tokens[1]).expect("invalid register")),
            "or" => Instruction::Or(Reg::from_str(tokens[1]).expect("invalid register")),
            "add" => Instruction::Add(Reg::from_str(tokens[1]).expect("invalid register")),
            "not" => Instruction::Not(Reg::from_str(tokens[1]).expect("invalid register")),
            "mul" => Instruction::Mul(Reg::from_str(tokens[1]).expect("invalid register")),
            "loadmem" => Instruction::LoadMem(Reg::from_str(tokens[1]).expect("invalid register")),
            "storemem" => Instruction::StoreMem(Reg::from_str(tokens[1]).expect("invalid register")),
            "loadlo" => Instruction::LoadLo(Target::parse(tokens[1])),
            "loadhi" => Instruction::LoadHi(Target::parse(tokens[1])),
            "jmp" => Instruction::Jmp(Target::parse(tokens[1])),
            "jz" => Instruction::Jz(Target::parse(tokens[1])),
            "jnz" => Instruction::Jnz(Target::parse(tokens[1])),
            _ => panic!("unknown opcode {}", tokens[0])
        }
    }
}

#[derive(Debug)]
pub enum Line {
    Label(String),
    Comment(String),
    Instruction(Instruction),
    Macro(String, Vec<Instruction>)
}

impl Line {
    pub fn parse(line: String) -> Line {
        match line.chars().next() {
            Some('#') | None => { 
                Line::Comment(line)
            },
            Some(':') => {
                Line::Label(line)
            },
            Some(_) => {
                let mut instructions = Vec::new();

                // handle macros
                let tokens : Vec<&str> = line.split_whitespace().collect();
                match tokens[0].to_lowercase().as_ref() {
                    "push" => {
                        let reg = Reg::from_str(tokens[1]).expect("invalid register");
                        assert_ne!(reg, Reg::ACC);
                        instructions.push(Instruction::LoadLo(Target::Constant(0xF)));
                        instructions.push(Instruction::Add(Reg::SP));
                        instructions.push(Instruction::StoreReg(Reg::SP));
                        instructions.push(Instruction::LoadReg(reg));
                        instructions.push(Instruction::StoreMem(Reg::SP));
                    },
                    "pop" => {
                        let reg = Reg::from_str(tokens[1]).expect("invalid register");
                        assert_ne!(reg, Reg::ACC);
                        instructions.push(Instruction::LoadMem(Reg::SP));
                        instructions.push(Instruction::StoreReg(reg));
                        instructions.push(Instruction::LoadLo(Target::Constant(0x1)));
                        instructions.push(Instruction::Add(Reg::SP));
                        instructions.push(Instruction::StoreReg(Reg::SP));
                    },
                    "inc" => {
                        let reg = Reg::from_str(tokens[1]).expect("invalid register");
                        instructions.push(Instruction::LoadLo(Target::Constant(0x1)));
                        instructions.push(Instruction::Add(reg));
                        instructions.push(Instruction::StoreReg(reg));
                    },
                    "dec" => {
                        let reg = Reg::from_str(tokens[1]).expect("invalid register");
                        instructions.push(Instruction::LoadLo(Target::Constant(0xf)));
                        instructions.push(Instruction::Add(reg));
                        instructions.push(Instruction::StoreReg(reg));
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
                        instructions.push(Instruction::LoadLo(Target::Constant(0x1)));
                        instructions.push(Instruction::Add(Reg::SP));
                        instructions.push(Instruction::StoreReg(Reg::SP));
                    },
                    "ret" => {
                        instructions.push(Instruction::LoadMem(Reg::SP));
                        instructions.push(Instruction::StoreReg(Reg::PC));
                    },
                    "halt" => {
                        instructions.push(Instruction::LoadLo(Target::Constant(0xF)));
                        instructions.push(Instruction::StoreReg(Reg::PC));
                    },
                    _ => {
                        return Line::Instruction(Instruction::parse(&line));
                    }
                }
                
                Line::Macro(line, instructions)
            },
        }
    }
}

pub fn simulate(rom: &[Instruction], mut cycle_limit: usize) {
    let mut mem = [0u8; 256];
    let mut regs = [0u8; 8];

    println!("# begin simulation");

    while cycle_limit > 0 && regs[Reg::PC as usize] != 0xFF {
        assert_ne!(cycle_limit, 0);
        cycle_limit -= 1;

        let instruction = &rom[regs[Reg::PC as usize] as usize];
        print!("# PC:{:02x} {:?} {:?}", regs[Reg::PC as usize], regs, instruction);
        let mut bump_pc = true;
        match instruction {
            Instruction::LoadReg(r) => regs[Reg::ACC as usize] = regs[*r as usize],
            Instruction::StoreReg(r) => {
                if r == &Reg::PC {
                    bump_pc = false;
                }
                regs[*r as usize] = regs[Reg::ACC as usize];
            }
            Instruction::Xor(r) => regs[Reg::ACC as usize] ^= regs[*r as usize],
            Instruction::And(r) => regs[Reg::ACC as usize] &= regs[*r as usize],
            Instruction::Or(r) => regs[Reg::ACC as usize] |= regs[*r as usize],
            Instruction::Add(r) => {
                let sum = (regs[Reg::ACC as usize] as u16) + (regs[*r as usize] as u16);
                regs[Reg::ACC as usize] = (sum & 0xff) as u8;
                regs[Reg::FLAGS as usize] = (sum >> 8) as u8;
            },
            Instruction::Not(r) => regs[Reg::ACC as usize] = !regs[*r as usize],
            Instruction::Mul(r) => regs[Reg::ACC as usize] = regs[Reg::ACC as usize].wrapping_mul(regs[*r as usize]),
            Instruction::LoadMem(r) => regs[Reg::ACC as usize] = mem[regs[*r as usize] as usize],
            Instruction::StoreMem(r) => mem[regs[*r as usize] as usize] = regs[Reg::ACC as usize],
            Instruction::LoadLo(t) => match t {
                Target::Constant(c) => {
                    let mut c = *c as i16;
                    c <<= 12;
                    c >>= 12;
                    regs[Reg::ACC as usize] = (c & 0xff) as u8;
                },
                _ => unreachable!()
            },
            Instruction::LoadHi(t) => match t {
                Target::Constant(c) => {
                    regs[Reg::ACC as usize] &= 0x0F;
                    regs[Reg::ACC as usize] |= c << 4;
                },
                _ => unreachable!()
            },
            Instruction::Jmp(t) => match t {
                Target::Constant(c) => {
                    bump_pc = false;
                    let mut c = *c as i16;
                    c <<= 11;
                    c >>= 11;
                    c += regs[Reg::PC as usize] as i16;
                    regs[Reg::PC as usize] = (c & 0xff) as u8;
                },
                _ => unreachable!()
            },
            Instruction::Jz(t) => match t {
                Target::Constant(c) => {
                    if regs[Reg::ACC as usize] == 0 {
                        bump_pc = false;
                        let mut c = *c as i16;
                        c <<= 11;
                        c >>= 11;
                        c += regs[Reg::PC as usize] as i16;
                        regs[Reg::PC as usize] = (c & 0xff) as u8;
                    }
                },
                _ => unreachable!()
            },
            Instruction::Jnz(t) => match t {
                Target::Constant(c) => {
                    if regs[Reg::ACC as usize] != 0 {
                        bump_pc = false;
                        let mut c = *c as i16;
                        c <<= 11;
                        c >>= 11;
                        c += regs[Reg::PC as usize] as i16;
                        regs[Reg::PC as usize] = (c & 0xff) as u8;
                    }
                },
                _ => unreachable!()
            }
        }

        println!(" {:?}", regs);

        if bump_pc {
            regs[Reg::PC as usize] += 1;
        }
    }

    
    if cycle_limit == 0 {
        println!("# simulation timed out");
    } else {
        println!("# simulation completed");
    }
}