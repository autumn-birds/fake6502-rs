mod fake6502;
use fake6502::{CPU, Backplane};

use std::fs::File;
use std::io::prelude::*;

struct PlainBackplane {
    mem: [u8; std::u16::MAX as usize + 1],
}

impl Backplane for PlainBackplane {
    fn read(&self, address: u16) -> u8 {
        self.mem[address as usize]
    }
    fn write(&mut self, address: u16, value: u8) {
        self.mem[address as usize] = value;
    }

    fn each_instr(&mut self, c: &mut CPU) -> bool {
        true
    }
}

fn main() -> std::io::Result<()> {
    let mut memory = PlainBackplane { mem: [0 as u8; std::u16::MAX as usize + 1] };

    let mut file = File::open("test.bin")?;
    file.read_exact(&mut (memory.mem))?;

    let mut cpu = CPU::new();

    //cpu.exec(mem, tickcount: u32)
    cpu.pc = 0x0400;

    Ok(())
}
