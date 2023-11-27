/*
 * main.rs : simulate Caxton Foster's "blue" CPU
 *
 * Copyright (c) 2023 Charles Suresh <charles.suresh@gmail.com>
 * Original C version Copyright (c) @brainwagon (Mark VandeWettering)
 *
 * A straight forward port to Rust of the simulator described at:
 * https://brainwagon.org/2011/07/07/a-basic-simulator-for-caxton-fosters-blue-architecture/
 *
 * Coding this up took me way longer than the 15 minutes that @brainwagon
 * says he took to code this in C, despite the fact that this is "just a port"
 * from existing C code. I'll make the excuse that it was probably because I'm
 * new to Rust - it took me quite a while to figure out how to do the
 * equivalent of getchar() in Rust - not sure if I reinvented any wheels here.
 *
 * I also took a fair bit of time rewriting the machine code to get rid of the
 * 16-bit overflow that Rust detected (which C didn't).
 *
 * This should be recoded to use enums instead of bare opcodes but that seemed
 * to be overkill for code that no one, except probably me, will ever use.
 */

struct State {
    pc: usize,				// Program Counter
    ir: u16,				// Instruction Register
    cs: u16,				// Condition/Status Register
    acc: u16,				// Accumulator
    head: usize,			// "Paper tape head" pointer
    mem: Vec<u16>,			// Memory
}

use std::io;

fn main() {
    let mut tape = String::new();
    let mut state = State {
        pc: 0,
        ir: 0,
        cs: 0,
        acc: 0,
        head: 0,
        mem: Vec::new(),
    };
    let rom: Vec<u16> = vec![		// machine code to print "Hello World"
        0x600C,
        0x100B,
        0x900A,
        0x700C,
        0x6007,
        0x100B,
        0x7007,
        0x600C,
        0xC000,
        0xA000,
        0x0000,
        0x0001,
    ];
    let data = "Hello World!\n";	// DATA segment
    for c in rom {			// ROM "bootloader"
        state.mem.push(c);
    }
    state.mem.push((0x7fff - data.len()) as u16); // 0x7fff is MAXINT in 16 bits
    for c in data.bytes() {		// DATA segment initialization
        state.mem.push(c as u16);
    }
    loop {
        state.fetch();
        state.execute(&mut tape);
        if state.ir == 0 {
            break;
        }
    }
}

impl State {
    pub fn fetch(&mut self) {
        self.ir = self.mem[self.pc];
        // println!("{:x}: {:x}",self.pc,self.ir);
        self.pc += 1;
    }

    pub fn execute(&mut self, mut s: &mut String) {
        let addr: usize = (self.ir & 0xfff).into();
        let op: u8 = (self.ir >> 12) as u8;
        // println!("op: {:x} addr: {:x} acc:{:x}",op, addr, self.acc);
        match op {
            0 => {}					// HLT
            1 => self.acc += self.mem[addr],		// ADD
            2 => self.acc ^= self.mem[addr],		// XOR
            3 => self.acc &= self.mem[addr],		// AND
            4 => self.acc |= self.mem[addr],		// OR
            5 => self.acc ^= 0xFFFF,			// NOT
            6 => {					// LDA
                self.acc = self.mem[addr];
                //println!("Addr.. {:x} : {:x}",addr, self.acc);
            }
            7 => self.mem[addr] = self.acc,		// STA
            8 => {					// SRJ
                self.acc = (self.pc & 0xfff) as u16;
                self.pc = addr;
            }
            9 => {					// JMA
                if self.acc & 0x8000 == 0x8000 {
                    self.pc = addr;
                }
            }
            10 => self.pc = addr,			// JMP
            11 => {					// IN
                // Rust doesn't appear to have an equivalent of getchar()
                // roll my own since the world doesn't have enough misery.
                // Keep up with the times by pretending input is on paper tape
                if self.head == s.len() {
                        io::stdin().read_line(&mut s).expect("Tape read error");
                        self.head = 0;
                        if self.head == s.len() {
                                // Bail out on ^D ?
                                self.acc = 0;
                                return;
                        }
                }
                self.acc = s.as_bytes()[self.head] as u16;
                self.head += 1;
            }
            12 => {					// OUT
                let c: char = self.acc as u8 as char;
                print!("{}", c);
            }
            13 => {					// RAL
                if self.acc & 0x8000 == 0x8000 {
                    self.acc = (self.acc << 1) | 1;
                } else {
                    self.acc <<= 1;
                }
            }
            14 => self.acc = self.cs,			// CSA
            15 => {}					// NOP
            _ => {} // can't happen but Rust insists on 0 .. 2^8 full coverage.
        }
    }
}
