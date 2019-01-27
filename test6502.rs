mod fake6502;
use fake6502::{CPU, Backplane};

use std::fs::File;
use std::io::prelude::*;

const TEST_START_ADDR: u16 = 0x0400;

struct TestSystem {
    mem: [u8; std::u16::MAX as usize + 1],
    
    // The test suite indicates success and/or failure by looping, either back to
    // the beginning of its code or in an infinite loop elsewhere on failure. So,...
    // we'll try and detect that.
    last_addr: u16,
    start_addr: u16,
    // Are we in a 'trap'?
    in_trap: bool,
}

impl TestSystem {
    fn new() -> TestSystem {
        TestSystem {
            mem: [0 as u8; std::u16::MAX as usize + 1],
            last_addr: TEST_START_ADDR,
            start_addr: TEST_START_ADDR,
            in_trap: false
        }
    }
}

impl Backplane for TestSystem {
    fn read(&self, address: u16) -> u8 {
        self.mem[address as usize]
    }

    fn write(&mut self, address: u16, value: u8) {
        self.mem[address as usize] = value;
    }

    fn each_instr(&mut self, c: &mut CPU) -> bool {
        self.in_trap = (c.pc == self.last_addr || c.pc == self.start_addr);
        self.in_trap
    }
}

fn main() -> std::io::Result<()> {
    let mut sys = TestSystem::new();

    let mut file = File::open("test.bin")?;
    file.read_exact(&mut (sys.mem))?;

    let mut cpu = CPU::new();

    cpu.pc = TEST_START_ADDR;
    loop {
        cpu.exec(&mut sys, 20);
        if sys.in_trap {
            break;
        }
    }
    //cpu.exec(mem, tickcount: u32)

    Ok(())
}
