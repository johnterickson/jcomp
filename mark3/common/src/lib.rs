extern crate strum;
#[macro_use]
extern crate strum_macros;

use std::str::FromStr;

use std::num::Wrapping;

use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Display, EnumString, Eq, PartialEq, PartialOrd, Ord)]
#[strum(serialize_all = "lowercase")]
pub enum Reg {
    ACC,
    ADDR,
    FLAGS,
    SP,
    PC,
    UNKNOWN
}

#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    Label(String),
    Absolute(u8),
    Offset(u8),
}

impl Target {
    pub fn parse(s: &str) -> Target {
        match s.chars().next() {
            Some(':') => Target::Label(s.to_owned()),
            Some(_) => Target::Absolute(u8::from_str_radix(s, 16).expect("invalid hex")),
            None => panic!("argument needed.")
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StackOffset(u8);

impl StackOffset {
    pub fn top() -> StackOffset {
        StackOffset::new(0)
    }

    pub fn new(val: u8) -> StackOffset {
        assert!(val < 8);
        StackOffset(val as u8)
    }

    pub fn parse(s: &str) -> StackOffset {
        let val = u8::from_str_radix(s, 16).expect("invalid hex");
        StackOffset::new(val)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PushableInstruction {
    LoadLo(Target),
    LoadHi(Target),
    Add(StackOffset),
    Xor(StackOffset),
    Not(StackOffset),
    Or(StackOffset),
    And(StackOffset),
    Mul(StackOffset),
    LoadFromStack(StackOffset),
    LoadMem,
    LoadPc,
}

impl PushableInstruction {
    fn resolve_pushable(&self, pc: u8, labels: &BTreeMap<&String,u8>) -> PushableInstruction {
        match self {
            PushableInstruction::LoadLo(t) => match t {
                Target::Absolute(_) => self.clone(),
                Target::Offset(o) => PushableInstruction::LoadLo(Target::Absolute(pc + o)),
                Target::Label(l) => PushableInstruction::LoadLo(Target::Absolute(labels[l] & 0xf)),
            },
            PushableInstruction::LoadHi(t) => match t {
                Target::Absolute(_) => self.clone(),
                Target::Offset(o) => PushableInstruction::LoadHi(Target::Absolute(((pc + o) >> 4) & 0xf)),
                Target::Label(l) => PushableInstruction::LoadHi(Target::Absolute((labels[l] >> 4) & 0xf)),
            },
            _ => self.clone()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    StoreAddr,
    StoreMem,
    JmpAcc,
    Jmp(Target),
    Jz(Target),
    Jnz(Target),
    StoreToStack(StackOffset),
    Discard(StackOffset),
    // DiscardPop(StackOffset),
    Alloc(StackOffset),
    PopDiscard(StackOffset),
    WithPush(PushableInstruction),
    WithoutPush(PushableInstruction),
}

impl Instruction {

    pub fn with_push(push: bool, p: PushableInstruction) -> Instruction {
        if push {
            Instruction::WithPush(p)
        } else {
            Instruction::WithoutPush(p)
        }
    }

    pub fn get_size(&self) -> u8 {
        match self {
            Instruction::Jmp(_) => 2,
            Instruction::Jnz(_) => 2,
            Instruction::Jz(_) => 2,
            _ => 1,
        }
    }

    pub fn encode(&self) -> (u8,Option<u8>) {
        match self {
            Instruction::Jmp(t) => (0x3c, match t { 
                Target::Absolute(c) => Some(*c),
                _ => unreachable!(),
            }),
            Instruction::Jnz(t) => (0x3d, match t { 
                Target::Absolute(c) => Some(*c),
                _ => unreachable!(),
            }),
            Instruction::Jz(t) => (0x3e, match t { 
                Target::Absolute(c) => Some(*c),
                _ => unreachable!(),
            }),
            _ => (0, None)
        }
    }

    pub fn parse(line: &str) -> Instruction {
        let tokens : Vec<&str> = line.split_whitespace().collect();

        let pushable = match tokens[0].to_lowercase().as_ref() {
            "loadlo" => Some(PushableInstruction::LoadLo(Target::parse(tokens[1]))),
            "loadhi" => Some(PushableInstruction::LoadHi(Target::parse(tokens[1]))),
            "add" => Some(PushableInstruction::Add(StackOffset::parse(tokens[1]))),
            "xor" => Some(PushableInstruction::Xor(StackOffset::parse(tokens[1]))),
            "not" => Some(PushableInstruction::Not(StackOffset::parse(tokens[1]))),
            "or" => Some(PushableInstruction::Or(StackOffset::parse(tokens[1]))),
            "and" => Some(PushableInstruction::And(StackOffset::parse(tokens[1]))),
            "mul" => Some(PushableInstruction::Mul(StackOffset::parse(tokens[1]))),
            "loadfromstack" => Some(PushableInstruction::LoadFromStack(StackOffset::parse(tokens[1]))),
            "loadmem" => Some(PushableInstruction::LoadMem),
            "loadpc" => Some(PushableInstruction::LoadPc),
            _ => None,
        };

        if let Some(pushable) = pushable {
            return match tokens.last().unwrap() {
                &"push" => Instruction::WithPush(pushable),
                _ => Instruction::WithoutPush(pushable),
            };
        }

        match tokens[0].to_lowercase().as_ref() {
            "storeaddr" => Instruction::StoreAddr,
            "storemem" => Instruction::StoreMem,
            "jmpacc" => Instruction::JmpAcc,
            "jmp" => Instruction::Jmp(Target::parse(tokens[1])),
            "jz" => Instruction::Jz(Target::parse(tokens[1])),
            "jnz" => Instruction::Jnz(Target::parse(tokens[1])),
            "storetostack" => Instruction::StoreToStack(StackOffset::parse(tokens[1])),
            "discard" => Instruction::Discard(StackOffset::parse(tokens[1])),
            "popdiscard" => Instruction::PopDiscard(StackOffset::parse(tokens[1])),
            "alloc" => Instruction::Alloc(StackOffset::parse(tokens[1])),
            _ => panic!("unknown opcode {}", tokens[0])
        }
    }
}

#[derive(Clone, Copy, Debug, EnumString, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum Macro {
    Call,
    Ret,
    Halt
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
                    "call" => {
                        instructions.push(Instruction::WithoutPush(PushableInstruction::LoadLo(Target::Offset(4))));
                        instructions.push(Instruction::WithPush(PushableInstruction::LoadHi(Target::Offset(3))));
                        instructions.push(Instruction::Jmp(Target::Label(tokens[1].to_owned())));
                        //retuns here
                    },
                    "ret" => {
                        instructions.push(Instruction::PopDiscard(StackOffset(0)));
                        instructions.push(Instruction::JmpAcc);
                    },
                    "halt" => {
                        instructions.push(Instruction::Jmp(Target::Absolute(0xFF)));
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


trait Resolver {
    fn resolve(&self, pc: u8, labels: &BTreeMap<&String,u8>) -> Instruction;
}

impl Resolver for Instruction {
    fn resolve(&self, pc: u8, labels: &BTreeMap<&String,u8>) -> Instruction {
        match self {
            Instruction::WithoutPush(i) => Instruction::WithoutPush(i.resolve_pushable(pc, labels)),
            Instruction::WithPush(i) => Instruction::WithPush(i.resolve_pushable(pc, labels)),
            Instruction::Jmp(t) => match t {
                Target::Absolute(_) => self.clone(),
                Target::Offset(o) => Instruction::Jmp(Target::Absolute(pc + o)),
                Target::Label(l) => Instruction::Jmp(Target::Absolute(labels[l])),
            },
            Instruction::Jz(t) => match t {
                Target::Absolute(_) => self.clone(),
                Target::Offset(o) => Instruction::Jnz(Target::Absolute(pc + o)),
                Target::Label(l) => Instruction::Jz(Target::Absolute(labels[l]))
            },
            Instruction::Jnz(t) => match t {
                Target::Absolute(_) => self.clone(),
                Target::Offset(o) => Instruction::Jz(Target::Absolute(pc + o)),
                Target::Label(l) => Instruction::Jnz(Target::Absolute(labels[l]))
            },
            _ => self.clone()
        }
    }
}

pub fn assemble(lines: Vec<Line>) -> Vec<Instruction> {

    println!("v2.0 raw");

    for (i,line) in lines.iter().enumerate() {
        println!("# Line {}: {:?}", i, line);
    }

    let labels = {
        let mut labels = BTreeMap::new();
        let mut address : u8 = 0;
        for line in &lines {
            match line {
                Line::Instruction(i) => { address += i.get_size(); },
                Line::Label(l) => { 
                    if let Some(existing) = labels.insert(l, address) {
                        panic!("label {:?} already exists at {}!", l, existing);
                    }
                }
                Line::Comment(_) => {},
                Line::Macro(_, instructions) => { 
                    for i in instructions {
                        address += i.get_size();
                    }
                }
            }
        }
        labels
    };


    for l in &labels {
        println!("# {:?}", l);
    }

    let rom = {
        let mut instructions = Vec::new();
        let mut pc = 0;

        let mut resolve_instruction = |i: &Instruction| {
            let resolved = i.resolve(pc, &labels);
            match resolved.encode() {
                (first,Some(second)) => {
                    print!("{:02x} {:02x} # @{:02x} {:?}", first, second, pc, i);
                },
                (first, None) => {
                    print!("{:02x} # @{:02x} {:?}", first, pc, i);
                }
            }

            if *i != resolved {
                print!(" {:?}", resolved);
            }
            println!();
            instructions.push(resolved);
            pc += i.get_size();
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

    rom
}

pub fn simulate(insts: &[Instruction], mut cycle_limit: usize) {
    let rom = {
        let mut rom = BTreeMap::new();
        let mut pc = 0;
        for i in insts {
            rom.insert(pc, i);
            pc += i.get_size();
        }
        rom
    };

    let mut mem = [Wrapping(0u8); 256];
    let mut regs = [Wrapping(0u8); 5];

    println!("# begin simulation");
    let mut cycles = 0;
    while cycle_limit != cycles && regs[Reg::PC as usize].0 != 0xFF {
        cycles += 1;

        let instruction = rom.get(&regs[Reg::PC as usize].0).expect("bad instruction address.");
        print!("# PC:{:02x} {:?}", regs[Reg::PC as usize], instruction);
        print!(" regs:{:?} stack:{:?}", regs, &mem[(regs[Reg::SP as usize].0 as usize)..]);

        let mut bump_pc = 1;
        match instruction {
            Instruction::WithPush(i) | Instruction::WithoutPush(i) => {
                match i {
                    PushableInstruction::LoadLo(t) => match t {
                        Target::Absolute(c) => {
                            let mut c = *c as i16;
                            c <<= 12;
                            c >>= 12;
                            regs[Reg::ACC as usize] = Wrapping((c & 0xff) as u8);
                        },
                        _ => unreachable!()
                    },
                    PushableInstruction::LoadHi(t) => match t {
                        Target::Absolute(c) => {
                            regs[Reg::ACC as usize].0 &= 0x0F;
                            regs[Reg::ACC as usize].0 |= c << 4;
                        },
                        _ => unreachable!()
                    },
                    PushableInstruction::Add(offset) => {
                        let stack_value = mem[regs[Reg::SP as usize].0 as usize + offset.0 as usize];
                        let sum = (regs[Reg::ACC as usize].0 as u16) + stack_value.0 as u16;
                        regs[Reg::ACC as usize] = Wrapping((sum & 0xff) as u8);
                        regs[Reg::FLAGS as usize] = Wrapping((sum >> 8) as u8);
                    },
                    PushableInstruction::Xor(offset) => regs[Reg::ACC as usize] ^= mem[regs[Reg::SP as usize].0 as usize + offset.0 as usize],
                    PushableInstruction::Not(offset) => regs[Reg::ACC as usize] = !mem[regs[Reg::SP as usize].0 as usize + offset.0 as usize],
                    PushableInstruction::Or(offset) => regs[Reg::ACC as usize] |= mem[regs[Reg::SP as usize].0 as usize + offset.0 as usize],
                    PushableInstruction::And(offset) => regs[Reg::ACC as usize] &= mem[regs[Reg::SP as usize].0 as usize + offset.0 as usize],
                    PushableInstruction::Mul(offset) => {
                        let stack_value = mem[regs[Reg::SP as usize].0 as usize + offset.0 as usize];
                        regs[Reg::ACC as usize] *= stack_value;
                    },
                    PushableInstruction::LoadFromStack(offset) => regs[Reg::ACC as usize] = mem[regs[Reg::SP as usize].0 as usize + offset.0 as usize],
                    PushableInstruction::LoadMem => regs[Reg::ACC as usize] = mem[regs[Reg::ADDR as usize].0 as usize],
                    PushableInstruction::LoadPc => regs[Reg::ACC as usize] = regs[Reg::PC as usize],
                }

                if let Instruction::WithPush(_) = instruction {
                    regs[Reg::SP as usize] += Wrapping(0xff);
                    mem[regs[Reg::SP as usize].0 as usize] = regs[Reg::ACC as usize];
                }
            },
            Instruction::StoreAddr => {
                regs[Reg::ADDR as usize] = regs[Reg::ACC as usize];
            },
            Instruction::StoreMem => {
                mem[regs[Reg::ADDR as usize].0 as usize]= regs[Reg::ACC as usize];
            },
            Instruction::JmpAcc => {
                bump_pc = 0;
                regs[Reg::PC as usize] = regs[Reg::ACC as usize];
            },
            Instruction::Jmp(t) => match t {
                Target::Absolute(c) => {
                    bump_pc = 0;
                    regs[Reg::PC as usize] = Wrapping(*c);
                },
                _ => unreachable!()
            },
            Instruction::Jz(t) => match t {
                Target::Absolute(c) => {
                    if regs[Reg::ACC as usize].0 == 0 {
                        bump_pc = 0;
                        regs[Reg::PC as usize] = Wrapping(*c);
                    } else {
                        bump_pc = 2;
                    }
                },
                _ => unreachable!()
            },
            Instruction::Jnz(t) => match t {
                Target::Absolute(c) => {
                    if regs[Reg::ACC as usize].0 != 0 {
                        bump_pc = 0;
                        regs[Reg::PC as usize] = Wrapping(*c);
                    } else {
                        bump_pc = 2;
                    }
                },
                _ => unreachable!()
            },
            Instruction::StoreToStack(offset) => {
                mem[(regs[Reg::SP as usize] + Wrapping(offset.0)).0 as usize] = regs[Reg::ACC as usize];
            },
            Instruction::Discard(offset) => {
                regs[Reg::SP as usize] += Wrapping(offset.0);
            },
            Instruction::Alloc(offset) => {
                regs[Reg::SP as usize] -= Wrapping(offset.0);
            },
            Instruction::PopDiscard(offset) => {
                regs[Reg::ACC as usize] = mem[regs[Reg::SP as usize].0 as usize];
                regs[Reg::SP as usize] += Wrapping(offset.0 + 1);
            }
            
        }

        println!(" regs:{:?} stack:{:?}", regs, &mem[(regs[Reg::SP as usize].0 as usize)..]);
        regs[Reg::PC as usize] += Wrapping(bump_pc);

    }

    
    if cycle_limit == cycles {
        println!("# simulation timed out after {} cycles", cycles);
    } else {
        println!("# simulation completed after {} cycles", cycles);
    }
}

