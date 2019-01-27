#![allow(dead_code)]
/* Fake6502 CPU emulator core v1.1 *******************
 * (c)2011 Mike Chambers (miker00lz@gmail.com)       *
 *****************************************************
 * v1.1 - Small bugfix in BIT opcode, but it was the *
 *        difference between a few games in my NES   *
 *        emulator working and being broken!         *
 *        I went through the rest carefully again    *
 *        after fixing it just to make sure I didn't *
 *        have any other typos! (Dec. 17, 2011)      *
 *                                                   *
 * v1.0 - First release (Nov. 24, 2011)              *
 *****************************************************
 * LICENSE: This source code is released into the    *
 * public domain, but if you use it please do give   *
 * credit. I put a lot of effort into writing this!  *
 *                                                   *
 *****************************************************
 * Fake6502 is a MOS Technology 6502 CPU emulation   *
 * engine in C. It was written as part of a Nintendo *
 * Entertainment System emulator I've been writing.  *
 *                                                   *
 * A couple important things to know about are two   *
 * defines in the code. One is "UNDOCUMENTED" which, *
 * when defined, allows Fake6502 to compile with     *
 * full support for the more predictable             *
 * undocumented instructions of the 6502. If it is   *
 * undefined, undocumented opcodes just act as NOPs. *
 *                                                   *
 * The other define is "NES_CPU", which causes the   *
 * code to compile without support for binary-coded  *
 * decimal (BCD) support for the ADC and SBC         *
 * opcodes. The Ricoh 2A03 CPU in the NES does not   *
 * support BCD, but is otherwise identical to the    *
 * standard MOS 6502. (Note that this define is      *
 * enabled in this file if you haven't changed it    *
 * yourself. If you're not emulating a NES, you      *
 * should comment it out.)                           *
 *                                                   *
 * If you do discover an error in timing accuracy,   *
 * or operation in general please e-mail me at the   *
 * address above so that I can fix it. Thank you!    *
 *                                                   *
 *****************************************************
 * Usage:                                            *
 *                                                   *
 * Fake6502 requires you to provide two external     *
 * functions:                                        *
 *                                                   *
 * uint8_t read6502(uint16_t address)                *
 * void write6502(uint16_t address, uint8_t value)   *
 *                                                   *
 * You may optionally pass Fake6502 the pointer to a *
 * function which you want to be called after every  *
 * emulated instruction. This function should be a   *
 * void with no parameters expected to be passed to  *
 * it.                                               *
 *                                                   *
 * This can be very useful. For example, in a NES    *
 * emulator, you check the number of clock ticks     *
 * that have passed so you can know when to handle   *
 * APU events.                                       *
 *                                                   *
 * To pass Fake6502 this pointer, use the            *
 * hookexternal(void *funcptr) function provided.    *
 *                                                   *
 * To disable the hook later, pass NULL to it.       *
 *****************************************************
 * Useful functions in this emulator:                *
 *                                                   *
 * void reset6502()                                  *
 *   - Call this once before you begin execution.    *
 *                                                   *
 * void exec6502(uint32_t tickcount)                 *
 *   - Execute 6502 code up to the next specified    *
 *     count of clock ticks.                         *
 *                                                   *
 * void step6502()                                   *
 *   - Execute a single instrution.                  *
 *                                                   *
 * void irq6502()                                    *
 *   - Trigger a hardware IRQ in the 6502 core.      *
 *                                                   *
 * void nmi6502()                                    *
 *   - Trigger an NMI in the 6502 core.              *
 *                                                   *
 * void hookexternal(void *funcptr)                  *
 *   - Pass a pointer to a void function taking no   *
 *     parameters. This will cause Fake6502 to call  *
 *     that function once after each emulated        *
 *     instruction.                                  *
 *                                                   *
 *****************************************************
 * Useful variables in this emulator:                *
 *                                                   *
 * uint32_t clockticks6502                           *
 *   - A running total of the emulated cycle count.  *
 *                                                   *
 * uint32_t instructions                             *
 *   - A running total of the total emulated         *
 *     instruction count. This is not related to     *
 *     clock cycle timing.                           *
 *                                                   *
 *****************************************************/

//#include <stdio.h>
//#include <stdint.h>

//6502 defines
//#define UNDOCUMENTED //when this is defined, undocumented opcodes are handled.
                     //otherwise, they're simply treated as NOPs.

//#define NES_CPU      //when this is defined, the binary-coded decimal (BCD)
                     //status flag is not honored by ADC and SBC. the 2A03
                     //CPU in the Nintendo Entertainment System does not
                     //support BCD operation.

use std::num::Wrapping;

const UNDOCUMENTED: bool = false;
const NES_CPU: bool = false;

//#define FLAG_CARRY     0x01
//#define FLAG_ZERO      0x02
//#define FLAG_INTERRUPT 0x04
//#define FLAG_DECIMAL   0x08
//#define FLAG_BREAK     0x10
//#define FLAG_CONSTANT  0x20
//#define FLAG_OVERFLOW  0x40
//#define FLAG_SIGN      0x80

// TODO: Adopt the nicer syntax from the sprocket-nes project
const FLAG_CARRY:       u8 = 0x01;
const FLAG_ZERO:        u8 = 0x02;
const FLAG_INTERRUPT:   u8 = 0x04;
const FLAG_DECIMAL:     u8 = 0x08;
const FLAG_BREAK:       u8 = 0x10;
const FLAG_CONSTANT:    u8 = 0x20;
const FLAG_OVERFLOW:    u8 = 0x40;
const FLAG_SIGN:        u8 = 0x80;

//#define BASE_STACK     0x100
/* I'm guessing at the type here... */
const BASE_STACK: u16 = 0x100;

//6502 CPU registers
//uint16_t pc;
//uint8_t sp, a, x, y, status;

//helper variables
//uint32_t instructions = 0; //keep track of total instructions executed
//uint32_t clockticks6502 = 0, clockgoal6502 = 0;
//uint16_t oldpc, ea, reladdr, value, result;
//uint8_t opcode, oldstatus;

pub struct CPU {
    /* 6502 CPU registers: */
    pub pc: Wrapping<u16>,
    pub sp: Wrapping<u8>,
    pub a: Wrapping<u8>,
    pub x: Wrapping<u8>,
    pub y: Wrapping<u8>,
    pub status: u8,

    /* Helper variables: */
    pub instructions_ran: u32,
    pub clockticks: u32,
    clockgoal: u32,
    // This variable is here because some C code tests whether the current addressing mode is 'acc'
    // (accumulator) by checking against the function lookup tables the C version of this emulator
    // used. Since I opted to turn those into a match in the Rust version, I needed a different way
    // to implement the same conditional.
    addr_acc: bool,
    // Some of these, perhaps especially 'result', are probably not needed in the struct, but for a
    // direct port, it's easiest, perhaps, to start out not trying to reason about whether a
    // variable's state being carried over between calls will matter.
    oldpc: Wrapping<u16>,
    // EA = Effective Address? This one *is* needed: Addressing modes are implemented as functions
    // that read a byte and set this value to the appropriate address.
    ea: Wrapping<u16>,
    reladdr: Wrapping<u16>,
    //uint8_t penaltyop, penaltyaddr;
    penaltyop: u8,
    penaltyaddr: u8,
    value: Wrapping<u16>,
    result: Wrapping<u16>,
    opcode: u8,
    oldstatus: u8,

    // Call the Backplane::each_instr() method after every instruction?
    pub do_callback: bool,
}

//externally supplied functions
//extern uint8_t read6502(uint16_t address);
//extern void write6502(uint16_t address, uint8_t value);

pub trait Backplane {
    // It might make sense to just use a &mut [u8] for example, but I feel like there's probably a
    // reason the original code did it this way: Any special behavior or mappings for special
    // memory addresses you want to have in the callback function, you can have.  (Consider e.g.
    // real systems where writing to a particular address actually controlled hardware.)
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);

    fn each_instr(&mut self, cpu: &mut CPU) -> bool;
}

impl CPU {
    // You're going to notice that nearly all of these functions take a 'mem' value that must imply
    // trait Backplane as an argument argument alongside the CPU struct.
    //
    // This may prove to have been a mistake, or may not. I felt that it would allow for more
    // flexibility on the caller's part and perhaps less borrowing tangles if we do not require the
    // CPU struct to own its memory.

    pub fn new() -> CPU {
        CPU {
            pc: Wrapping(0),
            sp: Wrapping(0xFD),
            a: Wrapping(0),
            x: Wrapping(0),
            y: Wrapping(0),
            status: Wrapping(0),
            addr_acc: false,
            instructions_ran: 0,
            clockticks: 0,
            clockgoal: 0,
            oldpc: Wrapping(0),
            ea: Wrapping(0),
            reladdr: Wrapping(0),
            penaltyaddr: 0,
            penaltyop: 0,
            value: Wrapping(0),
            result: Wrapping(0),
            opcode: 0,
            oldstatus: 0,
            do_callback: true
        }
    }

    // TODO: Figure out how to deal with the overflow problem. In C, it would have wrapped around,
    // I think; in Rust, it panics. Wrap-around is probably more authentic. We're going to run into
    // this later on as well.
    //
    // Thing of note: Std::num::Wrapping can be put around a u8, u16, etc., to make arithmetic on
    // that type wrap on overflow. That may be what we want if we want to mimick C's semantics.
    // Which would probably come closest to duplicating the behavior of the C source.
    //
    // (I found out it panics only in debug mode. But that definitely doesn't mean we should
    // just always run in release mode.)

    // TODO: A lot of functions would like to "read the byte under pc and advance pc."  We should
    // probably make that a small utility function.

    //a few general functions used by various other functions
    //void push16(uint16_t pushval) {
    //    write6502(BASE_STACK + sp, (pushval >> 8) & 0xFF);
    //    write6502(BASE_STACK + ((sp - 1) & 0xFF), pushval & 0xFF);
    //    sp -= 2;
    //}
    fn push16<T: Backplane>(&mut self, mem: &mut T, pushval: u16) {
        mem.write(BASE_STACK + (self.sp as u16), ((pushval >> 8) & 0x00FF) as u8);
        self.sp -= 1;
        mem.write(BASE_STACK + (self.sp as u16), (pushval & 0x00FF) as u8);
        self.sp -= 1;
    }

    //void push8(uint8_t pushval) {
    //    write6502(BASE_STACK + sp--, pushval);
    //}
    fn push8<T: Backplane>(&mut self, mem: &mut T, pushval: u8) {
        mem.write(BASE_STACK + (self.sp as u16), pushval);
        self.sp -= 1;
    }

    //uint16_t pull16() {
    //    uint16_t temp16;
    //    temp16 = read6502(BASE_STACK + ((sp + 1) & 0xFF)) | ((uint16_t)read6502(BASE_STACK + ((sp + 2) & 0xFF)) << 8);
    //    sp += 2;
    //    return(temp16);
    //}
    fn pull16<T: Backplane>(&mut self, mem: &T) -> u16 {
        let mut val: u16 = mem.read(BASE_STACK + ((self.sp as u16 + 1) & 0x00FF)) as u16;
        val            |= (mem.read(BASE_STACK + ((self.sp as u16 + 2) & 0x00FF)) as u16) << 8;
        self.sp += 2;
        val
    }

    //uint8_t pull8() {
    //    return (read6502(BASE_STACK + ++sp));
    //}
    fn pull8<T: Backplane>(&mut self, mem: &T) -> u8 {
        let val = mem.read(BASE_STACK + (self.sp as u16));
        self.sp += 1;
        val
    }

    //void reset6502() {
    //    pc = (uint16_t)read6502(0xFFFC) | ((uint16_t)read6502(0xFFFD) << 8);
    //    a = 0;
    //    x = 0;
    //    y = 0;
    //    sp = 0xFD;
    //    status |= FLAG_CONSTANT;
    //}
    fn reset<T: Backplane>(&mut self, mem: &T) {
        self.pc = Wrapping(mem.read(0xFFFC) as u16 | ((mem.read(0xFFFD) as u16) << 8));
        self.a = Wrapping(0);
        self.x = Wrapping(0);
        self.y = Wrapping(0);
        self.sp = Wrapping(0xFD);
        self.status |= FLAG_CONSTANT;
    }


    //addressing mode functions, calculates effective addresses
    //static void imp() { //implied
    //}
    fn addr_implied<T: Backplane>(&mut self, _mem: &T) {
    }

    //static void acc() { //accumulator
    //}
    fn addr_accumulator<T: Backplane>(&mut self, _mem: &T) {
        self.addr_acc = true;
    }

    //static void imm() { //immediate
    //    ea = pc++;
    //}
    fn addr_immediate<T: Backplane>(&mut self, _mem: &T) {
        self.ea = Wrapping(self.pc as u16);
        self.pc = Wrapping(self.pc + Wrapping(1));
    }

    //static void zp() { //zero-page
    //    ea = (uint16_t)read6502((uint16_t)pc++);
    //}
    fn addr_zeropage<T: Backplane>(&mut self, mem: &T) {
        self.ea = mem.read(self.pc.0) as u16;
        self.pc = self.pc + Wrapping(1);
    }

    //static void zpx() { //zero-page,X
    //    ea = ((uint16_t)read6502((uint16_t)pc++) + (uint16_t)x) & 0xFF; //zero-page wraparound
    //}
    fn addr_zeropage_x<T: Backplane>(&mut self, mem: &T) {
        self.ea = Wrapping(mem.read(self.pc.0) as u16 + (self.x.0 as u16)) & 0x00FF;
        // ( the & 0x00FF thing for zero-page wraparound)
        self.pc = self.pc + Wrapping(1);
    }

    //static void zpy() { //zero-page,Y
    //    ea = ((uint16_t)read6502((uint16_t)pc++) + (uint16_t)y) & 0xFF; //zero-page wraparound
    //}
    fn addr_zeropage_y<T: Backplane>(&mut self, mem: &T) {
        self.ea = mem.read(self.pc.0) as u16 + (self.y.0 as u16 & 0x00FF);
        // ( the & 0x00FF thing maybe for zero-page wraparound? blehhh)
        self.pc = self.pc + Wrapping(1);
    }

    //static void rel() { //relative for branch ops (8-bit immediate value, sign-extended)
    //    reladdr = (uint16_t)read6502(pc++);
    //    if (reladdr & 0x80) reladdr |= 0xFF00;
    //}
    fn addr_relative_branch<T: Backplane>(&mut self, mem: &T) {
        self.reladdr = mem.read(self.pc.0) as u16;
        if self.reladdr & 0x0080 != 0 {
            self.reladdr |= 0xFF00;
        }
        self.pc = self.pc + Wrapping(1);
    }

    //static void abso() { //absolute
    //    ea = (uint16_t)read6502(pc) | ((uint16_t)read6502(pc+1) << 8);
    //    pc += 2;
    //}
    fn addr_absolute<T: Backplane>(&mut self, mem: &T) {
        self.ea = mem.read(self.pc.0) as u16;
        self.ea |= (mem.read(self.pc + 1) as u16) << 8;
        self.pc = self.pc + Wrapping(2);
    }

    //static void absx() { //absolute,X
    //    uint16_t startpage;
    //    ea = ((uint16_t)read6502(pc) | ((uint16_t)read6502(pc+1) << 8));
    //    startpage = ea & 0xFF00;
    //    ea += (uint16_t)x;

    //    if (startpage != (ea & 0xFF00)) { //one cycle penlty for page-crossing on some opcodes
    //        penaltyaddr = 1;
    //    }

    //    pc += 2;
    //}
    fn addr_absolute_x<T: Backplane>(&mut self, mem: &T) {
        let startpage: u16;
        self.ea = mem.read(self.pc.0) as u16 | (mem.read(self.pc + 1) as u16) << 8;
        startpage = self.ea & 0xFF00;
        self.ea += self.x.0 as u16;

        if startpage != (self.ea & 0xFF00) {
            // original source: "one cycle penalty for page-crossing on some opcodes"
            self.penaltyaddr = 1;
        }

        self.pc = self.pc + Wrapping(2);
    }

    //static void absy() { //absolute,Y
    //    uint16_t startpage;
    //    ea = ((uint16_t)read6502(pc) | ((uint16_t)read6502(pc+1) << 8));
    //    startpage = ea & 0xFF00;
    //    ea += (uint16_t)y;

    //    if (startpage != (ea & 0xFF00)) { //one cycle penlty for page-crossing on some opcodes
    //        penaltyaddr = 1;
    //    }

    //    pc += 2;
    //}
    fn addr_absolute_y<T: Backplane>(&mut self, mem: &T) {
        let startpage: u16;
        self.ea = mem.read(self.pc.0) as u16 | (mem.read(self.pc + 1) as u16) << 8;
        startpage = self.ea & 0xFF00;
        self.ea += self.y.0 as u16;

        if startpage != (self.ea & 0xFF00) {
            // original source: "one cycle penalty for page-crossing on some opcodes"
            self.penaltyaddr = 1;
        }

        self.pc = self.pc + Wrapping(2);
    }

    //static void ind() { //indirect
    //    uint16_t eahelp, eahelp2;
    //    eahelp = (uint16_t)read6502(pc) | (uint16_t)((uint16_t)read6502(pc+1) << 8);
    //    eahelp2 = (eahelp & 0xFF00) | ((eahelp + 1) & 0x00FF); //replicate 6502 page-boundary wraparound bug
    //    ea = (uint16_t)read6502(eahelp) | ((uint16_t)read6502(eahelp2) << 8);
    //    pc += 2;
    //}
    fn addr_indirect<T: Backplane>(&mut self, mem: &T) {
        let eahelp: u16;
        let eahelp2: u16;
        eahelp = mem.read(self.pc.0) as u16 | (mem.read(self.pc + 1) as u16) << 8;
        // original source: "replicate 6502 page-boundary wraparound bug"
        eahelp2 = (eahelp & 0xFF00) | ((eahelp + 1) & 0x00FF);
        self.ea = mem.read(eahelp) as u16 | (mem.read(eahelp2) as u16) << 8;
        self.pc = self.pc + Wrapping(2);
    }

    //static void indx() { // (indirect,X)
    //    uint16_t eahelp;
    //    eahelp = (uint16_t)(((uint16_t)read6502(pc++) + (uint16_t)x) & 0xFF); //zero-page wraparound for table pointer
    //    ea = (uint16_t)read6502(eahelp & 0x00FF) | ((uint16_t)read6502((eahelp+1) & 0x00FF) << 8);
    //}
    fn addr_indirect_x<T: Backplane>(&mut self, mem: &T) {
        let eahelp: u16;
        eahelp = (mem.read(self.pc.0) as u16 + self.x.0 as u16) & 0x00FF; // original: "zero-page wraparound for table"
        self.ea = mem.read(eahelp & 0x00FF) as u16 | (mem.read((eahelp + 1) & 0x00FF) as u16) << 8;
        self.pc = self.pc + Wrapping(1);
    }

    //static void indy() { // (indirect),Y
    //    uint16_t eahelp, eahelp2, startpage;
    //    eahelp = (uint16_t)read6502(pc++);
    //    eahelp2 = (eahelp & 0xFF00) | ((eahelp + 1) & 0x00FF); //zero-page wraparound
    //    ea = (uint16_t)read6502(eahelp) | ((uint16_t)read6502(eahelp2) << 8);
    //    startpage = ea & 0xFF00;
    //    ea += (uint16_t)y;

    //    if (startpage != (ea & 0xFF00)) { //one cycle penlty for page-crossing on some opcodes
    //        penaltyaddr = 1;
    //    }
    //}
    fn addr_indirect_y<T: Backplane>(&mut self, mem: &T) {
        let eahelp: u16 = mem.read(self.pc.0) as u16;
        self.pc = self.pc + Wrapping(1);
        let eahelp2: u16 = (eahelp & 0xFF00) | ((eahelp + 1) & 0x00FF); // original: "zero-page wraparound"
        self.ea = mem.read(eahelp) as u16 | ((mem.read(eahelp2) as u16) << 8);
        let startpage: u16 = self.ea & 0xFF00;
        self.ea += self.y.0 as u16;

        if startpage != (self.ea & 0xFF00) { // original: "one-cycle penalty for page crossing on some opcodes"
            self.penaltyaddr = 1;
        }
    }


    //static uint16_t getvalue() {
    //    if (addrtable[opcode] == acc) return((uint16_t)a);
    //        else return((uint16_t)read6502(ea));
    //}
    fn getvalue<T: Backplane>(&mut self, mem: &T) -> u16 {
        // But why is it u16...?
        if self.addr_acc {
            self.a.0 as u16
        } else {
            mem.read(self.ea) as u16
        }
    }

    //static uint16_t getvalue16() {
    //    return((uint16_t)read6502(ea) | ((uint16_t)read6502(ea+1) << 8));
    //}
    fn getvalue_16<T: Backplane>(&mut self, mem: &T) -> u16 {
        mem.read(self.ea) as u16 | ((mem.read(self.ea + 1) as u16) << 8)
    }

    //static void putvalue(uint16_t saveval) {
    //    if (addrtable[opcode] == acc) a = (uint8_t)(saveval & 0x00FF);
    //        else write6502(ea, (saveval & 0x00FF));
    //}
    fn putvalue<T: Backplane>(&mut self, mem: &mut T, saveval: u16) {
        if self.addr_acc {
            self.a = (saveval & 0x00FF) as u8;
        } else {
            mem.write(self.ea, (saveval & 0x00FF) as u8);
        }
    }


    //flag modifier macros
    //#define setcarry() status |= FLAG_CARRY
    //#define clearcarry() status &= (~FLAG_CARRY)
    //#define setzero() status |= FLAG_ZERO
    //#define clearzero() status &= (~FLAG_ZERO)
    //#define setinterrupt() status |= FLAG_INTERRUPT
    //#define clearinterrupt() status &= (~FLAG_INTERRUPT)
    //#define setdecimal() status |= FLAG_DECIMAL
    //#define cleardecimal() status &= (~FLAG_DECIMAL)
    //#define setoverflow() status |= FLAG_OVERFLOW
    //#define clearoverflow() status &= (~FLAG_OVERFLOW)
    //#define setsign() status |= FLAG_SIGN
    //#define clearsign() status &= (~FLAG_SIGN)

    // This *seems* less efficient since they're actual functions. But I'm hoping the compiler is
    // scary-smart and might just realise it might not need an actual function call and return for
    // these, or whatever other optimization tricks it has up its sleeve. Maybe calling them on
    // const values also helps? I dunno.
    //
    // We could just write it out by hand every time. But these probably don't cost *too* much.
    fn flagset(&mut self, flag: u8) {
        self.status |= flag;
    }

    fn flagclear(&mut self, flag: u8) {
        self.status &= !flag;
    }

    //flag calculation macros
    //#define zerocalc(n) {\
    //    if ((n) & 0x00FF) clearzero();\
    //        else setzero();\
    //}

    //#define signcalc(n) {\
    //    if ((n) & 0x0080) setsign();\
    //        else clearsign();\
    //}

    //#define carrycalc(n) {\
    //    if ((n) & 0xFF00) setcarry();\
    //        else clearcarry();\
    //}

    //#define overflowcalc(n, m, o) { /* n = result, m = accumulator, o = memory */ \
    //    if (((n) ^ (uint16_t)(m)) & ((n) ^ (o)) & 0x0080) setoverflow();\
    //        else clearoverflow();\
    //}

    // These functions help with setting CPU flags to an appropriate value based on the result of
    // an operation. (For example, setting the overflow flag appropriately after some arithmetic is
    // done.) They were originally macros, but functions will probably work just as well.
    fn flagcalc_zero(&mut self, n: u16) {
        if (n & 0x00FF) != 0 {
            self.flagclear(FLAG_ZERO);
        } else {
            self.flagset(FLAG_ZERO);
        }
    }

    fn flagcalc_sign(&mut self, n: u16) {
        if (n & 0x0080) != 0 {
            self.flagset(FLAG_SIGN);
        } else {
            self.flagclear(FLAG_SIGN);
        }
    }

    fn flagcalc_carry(&mut self, n: u16) {
        if (n & 0xFF00) != 0 {
            self.flagset(FLAG_CARRY);
        } else {
            self.flagclear(FLAG_CARRY);
        }
    }

    fn flagcalc_overflow(&mut self, n: Wrapping<u16>, m: Wrapping<u8>, o: Wrapping<u16>) {
        // (The ^ is not exponentiation but bitwise XOR. Most people probably know this.)
        if (n ^ Wrapping(m as u16)) & ((n ^ o) & Wrapping(0x0080)) != 0 {
            self.flagset(FLAG_OVERFLOW);
        } else {
            self.flagclear(FLAG_OVERFLOW);
        }
    }

    //#define saveaccum(n) a = (uint8_t)((n) & 0x00FF)
    fn save_accumulator(&mut self, n: Wrapping<u16>) {
        self.a = (n & 0x00FF) as u8;
    }


    //instruction handler functions
    //static void adc() {
    fn inst_adc<T: Backplane>(&mut self, mem: &mut T) {
    //    penaltyop = 1;
        self.penaltyop = 1;
    //    value = getvalue();
        self.value = self.getvalue(mem);
    //    result = (uint16_t)a + value + (uint16_t)(status & FLAG_CARRY);
        self.result = self.a.0 as u16 + self.value + (self.status & FLAG_CARRY) as u16;
    //    carrycalc(result);
    //    zerocalc(result);
    //    overflowcalc(result, a, value);
    //    signcalc(result);
        // I copy these into variables because otherwise the borrow checker complains that I'm
        // trying to use a value (e.g., self.result) I've already borrowed (by calling, e.g.,
        // self.flagcalc_carry). It feels like there ought to be a less dumb way to do it, but for
        // now this at least compiles...
        let (r, a, v) = (self.result.0, self.a.0, self.value.0);
        self.flagcalc_carry(r);
        self.flagcalc_zero(r);
        self.flagcalc_overflow(r, a, v);
        self.flagcalc_sign(r);
        
        // I don't *really* want an actual if statement to be compiled here but I decided to
        // continue in the optimistic assumption that the Rust compiler and/or LLVM will notice
        // it's a const value and optimize it away.
    //    #ifndef NES_CPU
        if !NES_CPU {
    //    if (status & FLAG_DECIMAL) {
    //        clearcarry();
            
    //        if ((a & 0x0F) > 0x09) {
    //            a += 0x06;
    //        }
    //        if ((a & 0xF0) > 0x90) {
    //            a += 0x60;
    //            setcarry();
    //        }
            
    //        clockticks6502++;
    //    }
    //    #endif
            if self.status & FLAG_DECIMAL != 0 {
                self.flagclear(FLAG_CARRY);

                if (self.a.0 & 0x0F) > 0x09 {
                    self.a += Wrapping(0x06);
                }
                if (self.a.0 & 0xF0) > 0x90 {
                    self.a += Wrapping(0x60);
                    self.flagset(FLAG_CARRY);
                }

                self.clockticks += 1;
            }
        }
       
    //    saveaccum(result);
    //}
        let r = self.result;
        self.save_accumulator(r);
    }

    //static void and() {
    fn inst_and<T: Backplane>(&mut self, mem: &mut T) {
    //    penaltyop = 1;
    //    value = getvalue();
    //    result = (uint16_t)a & value;
        self.penaltyop = 1;
        self.value = self.getvalue(mem);
        self.result = (self.a.0 as u16) & self.value;
       
    //    zerocalc(result);
    //    signcalc(result);
        let r = self.result;
        self.flagcalc_zero(r);
        self.flagcalc_sign(r);
       
    //    saveaccum(result);
        self.save_accumulator(r);
    //}
    }

    //static void asl() {
    fn inst_asl<T: Backplane>(&mut self, mem: &mut T) {
    //    value = getvalue();
    //    result = value << 1;
        self.value = self.getvalue(mem);
        self.result = self.value << 1;

    //    carrycalc(result);
    //    zerocalc(result);
    //    signcalc(result);
        let r = self.result;
        self.flagcalc_carry(r);
        self.flagcalc_zero(r);
        self.flagcalc_sign(r);
       
    //    putvalue(result);
    //}
        self.putvalue(mem, r);
    }

    //static void bcc() {
    fn inst_bcc<T: Backplane>(&mut self, _mem: &mut T) {
    //    if ((status & FLAG_CARRY) == 0) {
    //        oldpc = pc;
    //        pc += reladdr;
    //        if ((oldpc & 0xFF00) != (pc & 0xFF00)) clockticks6502 += 2; //check if jump crossed a page boundary
    //            else clockticks6502++;
    //    }
        if (self.status & FLAG_CARRY) == 0 {
            self.oldpc = self.pc;
            self.pc = self.pc + Wrapping(self.reladdr);
            if (self.oldpc & 0xFF00) != (self.pc & 0xFF00) {
                // original: "check if jump crossed a page boundary"
                self.clockticks += 2;
            } else {
                self.clockticks += 1;
            }
        }
    //}
    }

    //static void bcs() {
    fn inst_bcs<T: Backplane>(&mut self, _mem: &mut T) {
    //    if ((status & FLAG_CARRY) == FLAG_CARRY) {
    //        oldpc = pc;
    //        pc += reladdr;
    //        if ((oldpc & 0xFF00) != (pc & 0xFF00)) clockticks6502 += 2; //check if jump crossed a page boundary
    //            else clockticks6502++;
    //    }
    //}
        if (self.status & FLAG_CARRY) == FLAG_CARRY {
            self.oldpc = self.pc;
            self.pc = self.pc + Wrapping(self.reladdr);
            if (self.oldpc & 0xFF00) != (self.pc & 0xFF00) {
                // original: "check if jump crossed a page boundary"
                self.clockticks += 2;
            } else {
                self.clockticks += 1;
            }
        }
    }

    //static void beq() {
    fn inst_beq<T: Backplane>(&mut self, _mem: &mut T) {
    //    if ((status & FLAG_ZERO) == FLAG_ZERO) {
    //        oldpc = pc;
    //        pc += reladdr;
    //        if ((oldpc & 0xFF00) != (pc & 0xFF00)) clockticks6502 += 2; //check if jump crossed a page boundary
    //            else clockticks6502++;
    //    }
    //}
        if (self.status & FLAG_ZERO) == FLAG_ZERO {
            self.oldpc = self.pc;
            self.pc = self.pc + Wrapping(self.reladdr);
            if (self.oldpc & 0xFF00) != (self.pc & 0xFF00) {
                // original: "check if jump crossed a page boundary"
                self.clockticks += 2;
            } else {
                self.clockticks += 1;
            }
        }
    }

    //static void bit() {
    fn inst_bit<T: Backplane>(&mut self, mem: &mut T) {
    //    value = getvalue();
    //    result = (uint16_t)a & value;
       
    //    zerocalc(result);
    //    status = (status & 0x3F) | (uint8_t)(value & 0xC0);
    //}
        self.value = self.getvalue(mem);
        self.result = self.a.0 as u16 & self.value;

        let r = self.result;
        self.flagcalc_zero(r);
        self.status = (self.status & 0x3F) | (self.value & 0x00C0) as u8;
    }

    //static void bmi() {
    fn inst_bmi<T: Backplane>(&mut self, _mem: &mut T) {
    //    if ((status & FLAG_SIGN) == FLAG_SIGN) {
    //        oldpc = pc;
    //        pc += reladdr;
    //        if ((oldpc & 0xFF00) != (pc & 0xFF00)) clockticks6502 += 2; //check if jump crossed a page boundary
    //            else clockticks6502++;
    //    }
    //}
        if (self.status & FLAG_SIGN) == FLAG_SIGN {
            self.oldpc = self.pc;
            self.pc = self.pc + Wrapping(self.reladdr);
            if (self.oldpc & 0xFF00) != (self.pc & 0xFF00) {
                // original: "check if jump crossed a page boundary"
                self.clockticks += 2;
            } else {
                self.clockticks += 1;
            }
        }
    }

    //static void bne() {
    fn inst_bne<T: Backplane>(&mut self, _mem: &mut T) {
    //    if ((status & FLAG_ZERO) == 0) {
    //        oldpc = pc;
    //        pc += reladdr;
    //        if ((oldpc & 0xFF00) != (pc & 0xFF00)) clockticks6502 += 2; //check if jump crossed a page boundary
    //            else clockticks6502++;
    //    }
    //}
        if (self.status & FLAG_ZERO) == 0 {
            self.oldpc = self.pc;
            self.pc = self.pc + Wrapping(self.reladdr);
            if (self.oldpc & 0xFF00) != (self.pc & 0xFF00) {
                // original: "check if jump crossed a page boundary"
                self.clockticks += 2;
            } else {
                self.clockticks += 1;
            }
        }
    }

    //static void bpl() {
    fn inst_bpl<T: Backplane>(&mut self, _mem: &mut T) {
    //    if ((status & FLAG_SIGN) == 0) {
    //        oldpc = pc;
    //        pc += reladdr;
    //        if ((oldpc & 0xFF00) != (pc & 0xFF00)) clockticks6502 += 2; //check if jump crossed a page boundary
    //            else clockticks6502++;
    //    }
    //}
        if (self.status & FLAG_SIGN) == 0 {
            self.oldpc = self.pc;
            self.pc = self.pc + Wrapping(self.reladdr);
            if (self.oldpc & 0xFF00) != (self.pc & 0xFF00) {
                // original: "check if jump crossed a page boundary"
                self.clockticks += 2;
            } else {
                self.clockticks += 1;
            }
        }
    }

    //static void brk() {
    fn inst_brk<T: Backplane>(&mut self, mem: &mut T) {
    //    pc++;
    //    push16(pc); //push next instruction address onto stack
    //    push8(status | FLAG_BREAK); //push CPU status to stack
    //    setinterrupt(); //set interrupt flag
    //    pc = (uint16_t)read6502(0xFFFE) | ((uint16_t)read6502(0xFFFF) << 8);
    //}
        self.pc = self.pc + Wrapping(1);
        let (pc, stat) = (self.pc, self.status);
        self.push16(mem, pc); // original: "push next instruction address onto stack"
        self.push8(mem, stat | FLAG_BREAK); // original: "push CPU status to stack"
        self.flagset(FLAG_INTERRUPT);
        self.pc = mem.read(0xFFFE) as u16 | ((mem.read(0xFFFF) as u16) << 8);
    }

    //static void bvc() {
    fn inst_bvc<T: Backplane>(&mut self, _mem: &mut T) {
    //    if ((status & FLAG_OVERFLOW) == 0) {
    //        oldpc = pc;
    //        pc += reladdr;
    //        if ((oldpc & 0xFF00) != (pc & 0xFF00)) clockticks6502 += 2; //check if jump crossed a page boundary
    //            else clockticks6502++;
    //    }
    //}
        if (self.status & FLAG_OVERFLOW) == 0 {
            self.oldpc = self.pc;
            self.pc = self.pc + Wrapping(self.reladdr);
            if (self.oldpc & 0xFF00) != (self.pc & 0xFF00) {
                // original: "check if jump crossed a page boundary"
                self.clockticks += 2;
            } else {
                self.clockticks += 1;
            }
        }
    }

    //static void bvs() {
    fn inst_bvs<T: Backplane>(&mut self, _mem: &mut T) {
    //    if ((status & FLAG_OVERFLOW) == FLAG_OVERFLOW) {
    //        oldpc = pc;
    //        pc += reladdr;
    //        if ((oldpc & 0xFF00) != (pc & 0xFF00)) clockticks6502 += 2; //check if jump crossed a page boundary
    //            else clockticks6502++;
    //    }
    //}
        if (self.status & FLAG_OVERFLOW) == FLAG_OVERFLOW {
            self.oldpc = self.pc;
            self.pc = self.pc + Wrapping(self.reladdr);
            if (self.oldpc & 0xFF00) != (self.pc & 0xFF00) {
                // original: "check if jump crossed a page boundary"
                self.clockticks += 2;
            } else {
                self.clockticks += 1;
            }
        }
    }

    //static void clc() {
    //    clearcarry();
    //}
    fn inst_clc<T: Backplane>(&mut self, _mem: &mut T) {
        self.flagclear(FLAG_CARRY);
    }

    //static void cld() {
    //    cleardecimal();
    //}
    fn inst_cld<T: Backplane>(&mut self, _mem: &mut T) {
        self.flagclear(FLAG_DECIMAL);
    }

    //static void cli() {
    //    clearinterrupt();
    //}
    fn inst_cli<T: Backplane>(&mut self, _mem: &mut T) {
        self.flagclear(FLAG_INTERRUPT);
    }

    //static void clv() {
    //    clearoverflow();
    //}
    fn inst_clv<T: Backplane>(&mut self, _mem: &mut T) {
        self.flagclear(FLAG_OVERFLOW);
    }

    //static void cmp() {
    fn inst_cmp<T: Backplane>(&mut self, mem: &mut T) {
    //    penaltyop = 1;
        self.penaltyop = 1;
    //    value = getvalue();
        self.value = self.getvalue(mem);
    //    result = (uint16_t)a - value;
        self.result = self.a.0 as u16 - self.value;
       
    //    if (a >= (uint8_t)(value & 0x00FF)) setcarry();
    //        else clearcarry();
        if self.a >= (self.value & 0x00FF) as u8 {
            self.flagset(FLAG_CARRY);
        } else {
            self.flagclear(FLAG_CARRY);
        }
    //    if (a == (uint8_t)(value & 0x00FF)) setzero();
    //        else clearzero();
        if self.a == (self.value & 0x00FF) as u8 {
            self.flagset(FLAG_ZERO);
        } else {
            self.flagclear(FLAG_ZERO);
        }
    //    signcalc(result);
        let r = self.result;
        self.flagcalc_sign(r);
    }
    //}

    //static void cpx() {
    fn inst_cpx<T: Backplane>(&mut self, mem: &mut T) {
    //    value = getvalue();
        self.value = self.getvalue(mem);
    //    result = (uint16_t)x - value;
        self.result = self.x.0 as u16 - self.value;
       
    //    if (x >= (uint8_t)(value & 0x00FF)) setcarry();
    //        else clearcarry();
    //    if (x == (uint8_t)(value & 0x00FF)) setzero();
    //        else clearzero();
        if self.x >= (self.value & 0x00FF) as u8 {
            self.flagset(FLAG_CARRY);
        } else {
            self.flagclear(FLAG_CARRY);
        }

        if self.x == (self.value & 0x00FF) as u8 {
            self.flagset(FLAG_ZERO);
        } else {
            self.flagclear(FLAG_ZERO);
        }

    //    signcalc(result);
        let r = self.result;
        self.flagcalc_sign(r);
    //}
    }

    //static void cpy() {
    fn inst_cpy<T: Backplane>(&mut self, mem: &mut T) {
    //    value = getvalue();
    //    result = (uint16_t)y - value;
        self.value = self.getvalue(mem);
        self.result = self.y.0 as u16 - self.value;
       
    //    if (y >= (uint8_t)(value & 0x00FF)) setcarry();
    //        else clearcarry();
    //    if (y == (uint8_t)(value & 0x00FF)) setzero();
    //        else clearzero();
        if self.y >= (self.value & 0x00FF) as u8 {
            self.flagset(FLAG_CARRY);
        } else {
            self.flagclear(FLAG_CARRY);
        }

        if self.y == (self.value & 0x00FF) as u8 {
            self.flagset(FLAG_ZERO);
        } else {
            self.flagclear(FLAG_ZERO);
        }

    //    signcalc(result);
        let r = self.result;
        self.flagcalc_sign(r);
    //}
    }

    //static void dec() {
    fn inst_dec<T: Backplane>(&mut self, mem: &mut T) {
    //    value = getvalue();
    //    result = value - 1;
        self.value = self.getvalue(mem);
        self.result = self.value - 1;
       
    //    zerocalc(result);
    //    signcalc(result);
        let r = self.result;
        self.flagcalc_zero(r);
        self.flagcalc_sign(r);
       
    //    putvalue(result);
        self.putvalue(mem, r);
    //}
    }

    //static void dex() {
    fn inst_dex<T: Backplane>(&mut self, _mem: &mut T) {
    //    x--;
        self.x -= Wrapping(1);
       
    //    zerocalc(x);
    //    signcalc(x);
        let x = self.x.0 as u16;
        self.flagcalc_zero(x);
        self.flagcalc_sign(x);
    //}
    }

    //static void dey() {
    fn inst_dey<T: Backplane>(&mut self, _mem: &mut T) {
    //    y--;
        self.y -= Wrapping(1);
       
    //    zerocalc(y);
    //    signcalc(y);
        let y = self.y.0 as u16;
        self.flagcalc_zero(y);
        self.flagcalc_sign(y);
    //}
    }

    //static void eor() {
    fn inst_eor<T: Backplane>(&mut self, mem: &mut T) {
    //    penaltyop = 1;
        self.penaltyop = 1;
    //    value = getvalue();
    //    result = (uint16_t)a ^ value;
        self.value = self.getvalue(mem);
        self.result = self.a.0 as u16 ^ self.value;
       
    //    zerocalc(result);
    //    signcalc(result);
        let r = self.result;
        self.flagcalc_zero(r);
        self.flagcalc_sign(r);
       
    //    saveaccum(result);
        self.save_accumulator(r);
    //}
    }

    //static void inc() {
    fn inst_inc<T: Backplane>(&mut self, mem: &mut T) {
    //    value = getvalue();
    //    result = value + 1;
        self.value = self.getvalue(mem);
        self.result = self.value + 1;
       
    //    zerocalc(result);
    //    signcalc(result);
        let r = self.result;
        self.flagcalc_zero(r);
        self.flagcalc_sign(r);
       
    //    putvalue(result);
        self.putvalue(mem, r);
    //}
    }

    //static void inx() {
    fn inst_inx<T: Backplane>(&mut self, _mem: &mut T) {
    //    x++;
        self.x += 1;
       
    //    zerocalc(x);
    //    signcalc(x);
        let x = self.x.0 as u16;
        self.flagcalc_zero(x);
        self.flagcalc_sign(x);
    //}
    }

    //static void iny() {
    fn inst_iny<T: Backplane>(&mut self, _mem: &mut T) {
    //    y++;
        self.y += 1;
       
    //    zerocalc(y);
    //    signcalc(y);
        let y = self.y.0 as u16;
        self.flagcalc_zero(y);
        self.flagcalc_sign(y);
    //}
    }

    //static void jmp() {
    fn inst_jmp<T: Backplane>(&mut self, _mem: &mut T) {
    //    pc = ea;
        self.pc = self.ea;
    //}
    }

    //static void jsr() {
    fn inst_jsr<T: Backplane>(&mut self, mem: &mut T) {
    //    push16(pc - 1);
    //    pc = ea;
        let pc = self.pc - 1;
        self.push16(mem, pc);
        self.pc = self.ea;
    //}
    }

    //static void lda() {
    fn inst_lda<T: Backplane>(&mut self, mem: &mut T) {
    //    penaltyop = 1;
    //    value = getvalue();
    //    a = (uint8_t)(value & 0x00FF);
        self.penaltyop = 1;
        self.value = self.getvalue(mem);
        self.a = (self.value & 0x00FF) as u8;
       
    //    zerocalc(a);
    //    signcalc(a);
        let a = self.a.0 as u16;
        self.flagcalc_zero(a);
        self.flagcalc_sign(a);
    //}
    }

    //static void ldx() {
    fn inst_ldx<T: Backplane>(&mut self, mem: &mut T) {
    //    penaltyop = 1;
    //    value = getvalue();
    //    x = (uint8_t)(value & 0x00FF);
        self.penaltyop = 1;
        self.value = self.getvalue(mem);
        self.x = (self.value & 0x00FF) as u8;
       
    //    zerocalc(x);
    //    signcalc(x);
        let x = self.x.0 as u16;
        self.flagcalc_zero(x);
        self.flagcalc_sign(x);
    //}
    }

    //static void ldy() {
    fn inst_ldy<T: Backplane>(&mut self, mem: &mut T) {
    //    penaltyop = 1;
    //    value = getvalue();
    //    y = (uint8_t)(value & 0x00FF);
        self.penaltyop = 1;
        self.value = self.getvalue(mem);
        self.y = (self.value & 0x00FF) as u8;
       
    //    zerocalc(y);
    //    signcalc(y);
        let y = self.y.0 as u16;
        self.flagcalc_zero(y);
        self.flagcalc_sign(y);
    //}
    }

    //static void lsr() {
    fn inst_lsr<T: Backplane>(&mut self, mem: &mut T) {
    //    value = getvalue();
    //    result = value >> 1;
        self.value = self.getvalue(mem);
        self.result = self.value >> 1;
       
    //    if (value & 1) setcarry();
    //        else clearcarry();
        if self.value & 1 != 0 {
            self.flagset(FLAG_CARRY);
        } else {
            self.flagclear(FLAG_CARRY);
        }

    //    zerocalc(result);
    //    signcalc(result);
        let r = self.result;
        self.flagcalc_zero(r);
        self.flagcalc_sign(r);
       
    //    putvalue(result);
        self.putvalue(mem, r);
    //}
    }

    //static void nop() {
    fn inst_nop<T: Backplane>(&mut self, _mem: &mut T) {
    //    switch (opcode) {
        match self.opcode {
            0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
                self.penaltyop = 1;
            }
            _ => {}
        };
    //        case 0x1C:
    //        case 0x3C:
    //        case 0x5C:
    //        case 0x7C:
    //        case 0xDC:
    //        case 0xFC:
    //            penaltyop = 1;
    //            break;
    //    }
    //}
    }

    //static void ora() {
    fn inst_ora<T: Backplane>(&mut self, mem: &mut T) {
    //    penaltyop = 1;
    //    value = getvalue();
    //    result = (uint16_t)a | value;
        self.penaltyop = 1;
        self.value = self.getvalue(mem);
        self.result = self.a.0 as u16 | self.value;
       
    //    zerocalc(result);
    //    signcalc(result);
        let r = self.result;
        self.flagcalc_zero(r);
        self.flagcalc_sign(r);
       
    //    saveaccum(result);
        self.save_accumulator(r);
    //}
    }

    //static void pha() {
    fn inst_pha<T: Backplane>(&mut self, mem: &mut T) {
    //    push8(a);
        let a = self.a;
        self.push8(mem, a);
    //}
    }

    //static void php() {
    fn inst_php<T: Backplane>(&mut self, mem: &mut T) {
    //    push8(status | FLAG_BREAK);
        let s = self.status | FLAG_BREAK;
        self.push8(mem, s);
    //}
    }

    //static void pla() {
    fn inst_pla<T: Backplane>(&mut self, mem: &mut T) {
    //    a = pull8();
        self.a = self.pull8(mem);
       
    //    zerocalc(a);
    //    signcalc(a);
        let a = self.a.0 as u16;
        self.flagcalc_zero(a);
        self.flagcalc_sign(a);
    //}
    }

    //static void plp() {
    fn inst_plp<T: Backplane>(&mut self, mem: &mut T) {
    //    status = pull8() | FLAG_CONSTANT;
        self.status = self.pull8(mem) | FLAG_CONSTANT;
    //}
    }

    //static void rol() {
    fn inst_rol<T: Backplane>(&mut self, mem: &mut T) {
    //    value = getvalue();
    //    result = (value << 1) | (status & FLAG_CARRY);
        self.value = self.getvalue(mem);
        self.result = (self.value << 1) | (self.status & FLAG_CARRY) as u16;
       
    //    carrycalc(result);
    //    zerocalc(result);
    //    signcalc(result);
        let r = self.result;
        self.flagcalc_carry(r);
        self.flagcalc_zero(r);
        self.flagcalc_sign(r);
       
    //    putvalue(result);
        self.putvalue(mem, r);
    //}
    }

    //static void ror() {
    fn inst_ror<T: Backplane>(&mut self, mem: &mut T) {
    //    value = getvalue();
    //    result = (value >> 1) | ((status & FLAG_CARRY) << 7);
        self.value = self.getvalue(mem);
       
    //    if (value & 1) setcarry();
    //        else clearcarry();
    //    zerocalc(result);
    //    signcalc(result);
        if (self.value & 1) != 0 {
            self.flagset(FLAG_CARRY);
        } else {
            self.flagclear(FLAG_CARRY);
        }
       
    //    putvalue(result);
        let r = self.result;
        self.putvalue(mem, r);
    //}
    }

    //static void rti() {
    fn inst_rti<T: Backplane>(&mut self, mem: &mut T) {
    //    status = pull8();
    //    value = pull16();
    //    pc = value;
        self.status = self.pull8(mem);
        self.value = self.pull16(mem);
        self.pc = self.value;
    //}
    }

    //static void rts() {
    fn inst_rts<T: Backplane>(&mut self, mem: &mut T) {
    //    value = pull16();
    //    pc = value + 1;
        self.value = self.pull16(mem);
        self.pc = self.value + 1;
    //}
    }

    //static void sbc() {
    fn inst_sbc<T: Backplane>(&mut self, mem: &mut T) {
    //    penaltyop = 1;
    //    value = getvalue() ^ 0x00FF;
    //    result = (uint16_t)a + value + (uint16_t)(status & FLAG_CARRY);
        self.penaltyop = 1;
        self.value = self.getvalue(mem) ^ 0x00FF;
        self.result = Wrapping(self.a.0 as u16) + self.value + Wrapping((self.status & FLAG_CARRY) as u16);
       
    //    carrycalc(result);
    //    zerocalc(result);
    //    overflowcalc(result, a, value);
    //    signcalc(result);
        let (r, a, v) = (self.result.0, self.a.0, self.value.0);
        self.flagcalc_carry(r);
        self.flagcalc_zero(r);
        self.flagcalc_overflow(r, a, v);
        self.flagcalc_sign(r);

    //    #ifndef NES_CPU
        if !NES_CPU {
    //    if (status & FLAG_DECIMAL) {
    //        clearcarry();
            
    //        a -= 0x66;
    //        if ((a & 0x0F) > 0x09) {
    //            a += 0x06;
    //        }
    //        if ((a & 0xF0) > 0x90) {
    //            a += 0x60;
    //            setcarry();
    //        }
            
    //        clockticks6502++;
            if (self.status & FLAG_DECIMAL) != 0 {
                self.flagclear(FLAG_CARRY);

                // TODO: This is definitely going to overflow sometimes, we should probably have
                // figured out what to do earlier...
                self.a -= Wrapping(0x66);
                if (self.a.0 self.a & 0x0F) > 0x09 {
                    self.a += Wrapping(0x06);
                }
                if (self.a.0 self.a & 0xF0) > 0x90 {
                    self.a += Wrapping(0x60);
                    self.flagset(FLAG_CARRY);
                }

                self.clockticks += 1;
            }
    //    }
    //    #endif
        }
       
    //    saveaccum(result);
        let r = self.result;
        self.save_accumulator(r);
    //}
    }

    //static void sec() {
    fn inst_sec<T: Backplane>(&mut self, _mem: &mut T) {
    //    setcarry();
        self.flagset(FLAG_CARRY);
    //}
    }

    //static void sed() {
    fn inst_sed<T: Backplane>(&mut self, _mem: &mut T) {
    //    setdecimal();
        self.flagset(FLAG_DECIMAL);
    //}
    }

    //static void sei() {
    fn inst_sei<T: Backplane>(&mut self, _mem: &mut T) {
    //    setinterrupt();
        self.flagset(FLAG_INTERRUPT);
    //}
    }

    //static void sta() {
    fn inst_sta<T: Backplane>(&mut self, mem: &mut T) {
    //    putvalue(a);
        let a = self.a.0 as u16;
        self.putvalue(mem, a);
    //}
    }

    //static void stx() {
    fn inst_stx<T: Backplane>(&mut self, mem: &mut T) {
    //    putvalue(x);
        let x = self.x.0 as u16;
        self.putvalue(mem, x);
    //}
    }

    //static void sty() {
    fn inst_sty<T: Backplane>(&mut self, mem: &mut T) {
    //    putvalue(y);
        let y = self.y.0 as u16;
        self.putvalue(mem, y);
    //}
    }

    //static void tax() {
    fn inst_tax<T: Backplane>(&mut self, _mem: &mut T) {
    //    x = a;
        self.x = self.a;
       
    //    zerocalc(x);
    //    signcalc(x);
        let x = self.x.0 as u16;
        self.flagcalc_zero(x);
        self.flagcalc_sign(x);
    //}
    }

    //static void tay() {
    fn inst_tay<T: Backplane>(&mut self, _mem: &mut T) {
    //    y = a;
        self.y = self.a;
       
    //    zerocalc(y);
    //    signcalc(y);
        let y = self.y.0 as u16;
        self.flagcalc_zero(y);
        self.flagcalc_sign(y);
    //}
    }

    //static void tsx() {
    fn inst_tsx<T: Backplane>(&mut self, _mem: &mut T) {
    //    x = sp;
        self.x = self.sp;
       
    //    zerocalc(x);
    //    signcalc(x);
        let x = self.x.0 as u16;
        self.flagcalc_zero(x);
        self.flagcalc_sign(x);
    //}
    }

    //static void txa() {
    fn inst_txa<T: Backplane>(&mut self, _mem: &mut T) {
    //    a = x;
        self.a = self.x;
       
    //    zerocalc(a);
    //    signcalc(a);
        let a = self.a.0 as u16;
        self.flagcalc_zero(a);
        self.flagcalc_sign(a);
    //}
    }

    //static void txs() {
    fn inst_txs<T: Backplane>(&mut self, _mem: &mut T) {
    //    sp = x;
        self.sp = self.x;
    //}
    }

    //static void tya() {
    fn inst_tya<T: Backplane>(&mut self, _mem: &mut T) {
    //    a = y;
        self.a = self.y;
       
    //    zerocalc(a);
    //    signcalc(a);
        let a = self.a.0 as u16;
        self.flagcalc_zero(a);
        self.flagcalc_sign(a);
    //}
    }

    //undocumented instructions
    // TODO: Provide actual implementations.
    //#ifdef UNDOCUMENTED
    //    static void lax() {
    //        lda();
    //        ldx();
    //    }
    fn inst_lax<T: Backplane>(&mut self, mem: &mut T) {
        self.inst_nop(mem);
    }

    //    static void sax() {
    //        sta();
    //        stx();
    //        putvalue(a & x);
    //        if (penaltyop && penaltyaddr) clockticks6502--;
    //    }
    fn inst_sax<T: Backplane>(&mut self, mem: &mut T) {
        self.inst_nop(mem);
    }

    //    static void dcp() {
    //        dec();
    //        cmp();
    //        if (penaltyop && penaltyaddr) clockticks6502--;
    //    }
    fn inst_dcp<T: Backplane>(&mut self, mem: &mut T) {
        self.inst_nop(mem);
    }

    //    static void isb() {
    //        inc();
    //        sbc();
    //        if (penaltyop && penaltyaddr) clockticks6502--;
    //    }
    fn inst_isb<T: Backplane>(&mut self, mem: &mut T) {
        self.inst_nop(mem);
    }

    //    static void slo() {
    //        asl();
    //        ora();
    //        if (penaltyop && penaltyaddr) clockticks6502--;
    //    }
    fn inst_slo<T: Backplane>(&mut self, mem: &mut T) {
        self.inst_nop(mem);
    }

    //    static void rla() {
    //        rol();
    //        and();
    //        if (penaltyop && penaltyaddr) clockticks6502--;
    //    }
    fn inst_rla<T: Backplane>(&mut self, mem: &mut T) {
        self.inst_nop(mem);
    }

    //    static void sre() {
    //        lsr();
    //        eor();
    //        if (penaltyop && penaltyaddr) clockticks6502--;
    //    }
    fn inst_sre<T: Backplane>(&mut self, mem: &mut T) {
        self.inst_nop(mem);
    }

    //    static void rra() {
    //        ror();
    //        adc();
    //        if (penaltyop && penaltyaddr) clockticks6502--;
    //    }
     fn inst_rra<T: Backplane>(&mut self, mem: &mut T) {
        self.inst_nop(mem);
    }

    //#else
    //    #define lax nop
    //    #define sax nop
    //    #define dcp nop
    //    #define isb nop
    //    #define slo nop
    //    #define rla nop
    //    #define sre nop
    //    #define rra nop
    //#endif


    //void nmi6502() {
    //    push16(pc);
    //    push8(status);
    //    status |= FLAG_INTERRUPT;
    //    pc = (uint16_t)read6502(0xFFFA) | ((uint16_t)read6502(0xFFFB) << 8);
    //}

    //void irq6502() {
    //    push16(pc);
    //    push8(status);
    //    status |= FLAG_INTERRUPT;
    //    pc = (uint16_t)read6502(0xFFFE) | ((uint16_t)read6502(0xFFFF) << 8);
    //}

    //uint8_t callexternal = 0;
    //void (*loopexternal)();

    //void exec6502(uint32_t tickcount) {
    pub fn exec<T: Backplane>(&mut self, mem: &mut T, tickcount: u32) {
    //    clockgoal6502 += tickcount;
        self.clockgoal += tickcount;

    //    while (clockticks6502 < clockgoal6502) {
        while self.clockticks < self.clockgoal {
    //        opcode = read6502(pc++);
    //        status |= FLAG_CONSTANT;
            self.opcode = mem.read(self.pc.0);
            self.pc = self.pc + Wrapping(1);
            self.flagset(FLAG_CONSTANT);

    //        penaltyop = 0;
    //        penaltyaddr = 0;
            self.penaltyop = 0;
            self.penaltyaddr = 0;
            self.addr_acc = false;

    //        (*addrtable[opcode])();
    //        (*optable[opcode])();
    //        clockticks6502 += ticktable[opcode];
    //        if (penaltyop && penaltyaddr) clockticks6502++;
            self.clockticks += self.run_one_op(mem);
            if self.penaltyop != 0 && self.penaltyaddr != 0 {
                self.clockticks += 1;
            }

    //        instructions++;
            // TODO: If anything needed to watch out for overflows, it's this.
            self.instructions_ran += 1;

            // TODO: Figure out how a callback works. Maybe an Option<fn>?
    //        if (callexternal) (*loopexternal)();
            
            if self.do_callback {
                if !mem.each_instr(self) {
                    break;
                }
            }

    //    }
    //}
        }
    }

    fn run_one_op<T: Backplane>(&mut self, mem: &mut T) -> u32 {
        // Okay, so I decided to convert the tables of function pointers in the original into a
        // giant match statement after seeing a comment that stated LLVM would be able to optimize
        // said match statement into a jump table anyway, but unfortunately this means that we do
        // have a really tremendous match statement here. Try to think of it like a section of data
        // and you might feel better.
        //
        // (If you're changing code, please don't deviate from the pattern set out here at all.
        // Ideally, make sure the change agrees with the data in the json... I know, it's a hacky
        // system...)

        // We return the number of cycles running the instruction took (rather than looking it up
        // anywhere.)
        return match self.opcode {
            0   => { self.addr_implied(mem);           self.inst_brk(mem);   7 },
            1   => { self.addr_indirect_x(mem);        self.inst_ora(mem);   6 },
            2   => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            3   => { self.addr_indirect_x(mem);        self.inst_slo(mem);   8 },
            4   => { self.addr_zeropage(mem);          self.inst_nop(mem);   3 },
            5   => { self.addr_zeropage(mem);          self.inst_ora(mem);   3 },
            6   => { self.addr_zeropage(mem);          self.inst_asl(mem);   5 },
            7   => { self.addr_zeropage(mem);          self.inst_slo(mem);   5 },
            8   => { self.addr_implied(mem);           self.inst_php(mem);   3 },
            9   => { self.addr_immediate(mem);         self.inst_ora(mem);   2 },
            10  => { self.addr_accumulator(mem);       self.inst_asl(mem);   2 },
            11  => { self.addr_immediate(mem);         self.inst_nop(mem);   2 },
            12  => { self.addr_absolute(mem);          self.inst_nop(mem);   4 },
            13  => { self.addr_absolute(mem);          self.inst_ora(mem);   4 },
            14  => { self.addr_absolute(mem);          self.inst_asl(mem);   6 },
            15  => { self.addr_absolute(mem);          self.inst_slo(mem);   6 },
            16  => { self.addr_relative_branch(mem);   self.inst_bpl(mem);   2 },
            17  => { self.addr_indirect_y(mem);        self.inst_ora(mem);   5 },
            18  => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            19  => { self.addr_indirect_y(mem);        self.inst_slo(mem);   8 },
            20  => { self.addr_zeropage_x(mem);        self.inst_nop(mem);   4 },
            21  => { self.addr_zeropage_x(mem);        self.inst_ora(mem);   4 },
            22  => { self.addr_zeropage_x(mem);        self.inst_asl(mem);   6 },
            23  => { self.addr_zeropage_x(mem);        self.inst_slo(mem);   6 },
            24  => { self.addr_implied(mem);           self.inst_clc(mem);   2 },
            25  => { self.addr_absolute_y(mem);        self.inst_ora(mem);   4 },
            26  => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            27  => { self.addr_absolute_y(mem);        self.inst_slo(mem);   7 },
            28  => { self.addr_absolute_x(mem);        self.inst_nop(mem);   4 },
            29  => { self.addr_absolute_x(mem);        self.inst_ora(mem);   4 },
            30  => { self.addr_absolute_x(mem);        self.inst_asl(mem);   7 },
            31  => { self.addr_absolute_x(mem);        self.inst_slo(mem);   7 },
            32  => { self.addr_absolute(mem);          self.inst_jsr(mem);   6 },
            33  => { self.addr_indirect_x(mem);        self.inst_and(mem);   6 },
            34  => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            35  => { self.addr_indirect_x(mem);        self.inst_rla(mem);   8 },
            36  => { self.addr_zeropage(mem);          self.inst_bit(mem);   3 },
            37  => { self.addr_zeropage(mem);          self.inst_and(mem);   3 },
            38  => { self.addr_zeropage(mem);          self.inst_rol(mem);   5 },
            39  => { self.addr_zeropage(mem);          self.inst_rla(mem);   5 },
            40  => { self.addr_implied(mem);           self.inst_plp(mem);   4 },
            41  => { self.addr_immediate(mem);         self.inst_and(mem);   2 },
            42  => { self.addr_accumulator(mem);       self.inst_rol(mem);   2 },
            43  => { self.addr_immediate(mem);         self.inst_nop(mem);   2 },
            44  => { self.addr_absolute(mem);          self.inst_bit(mem);   4 },
            45  => { self.addr_absolute(mem);          self.inst_and(mem);   4 },
            46  => { self.addr_absolute(mem);          self.inst_rol(mem);   6 },
            47  => { self.addr_absolute(mem);          self.inst_rla(mem);   6 },
            48  => { self.addr_relative_branch(mem);   self.inst_bmi(mem);   2 },
            49  => { self.addr_indirect_y(mem);        self.inst_and(mem);   5 },
            50  => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            51  => { self.addr_indirect_y(mem);        self.inst_rla(mem);   8 },
            52  => { self.addr_zeropage_x(mem);        self.inst_nop(mem);   4 },
            53  => { self.addr_zeropage_x(mem);        self.inst_and(mem);   4 },
            54  => { self.addr_zeropage_x(mem);        self.inst_rol(mem);   6 },
            55  => { self.addr_zeropage_x(mem);        self.inst_rla(mem);   6 },
            56  => { self.addr_implied(mem);           self.inst_sec(mem);   2 },
            57  => { self.addr_absolute_y(mem);        self.inst_and(mem);   4 },
            58  => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            59  => { self.addr_absolute_y(mem);        self.inst_rla(mem);   7 },
            60  => { self.addr_absolute_x(mem);        self.inst_nop(mem);   4 },
            61  => { self.addr_absolute_x(mem);        self.inst_and(mem);   4 },
            62  => { self.addr_absolute_x(mem);        self.inst_rol(mem);   7 },
            63  => { self.addr_absolute_x(mem);        self.inst_rla(mem);   7 },
            64  => { self.addr_implied(mem);           self.inst_rti(mem);   6 },
            65  => { self.addr_indirect_x(mem);        self.inst_eor(mem);   6 },
            66  => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            67  => { self.addr_indirect_x(mem);        self.inst_sre(mem);   8 },
            68  => { self.addr_zeropage(mem);          self.inst_nop(mem);   3 },
            69  => { self.addr_zeropage(mem);          self.inst_eor(mem);   3 },
            70  => { self.addr_zeropage(mem);          self.inst_lsr(mem);   5 },
            71  => { self.addr_zeropage(mem);          self.inst_sre(mem);   5 },
            72  => { self.addr_implied(mem);           self.inst_pha(mem);   3 },
            73  => { self.addr_immediate(mem);         self.inst_eor(mem);   2 },
            74  => { self.addr_accumulator(mem);       self.inst_lsr(mem);   2 },
            75  => { self.addr_immediate(mem);         self.inst_nop(mem);   2 },
            76  => { self.addr_absolute(mem);          self.inst_jmp(mem);   3 },
            77  => { self.addr_absolute(mem);          self.inst_eor(mem);   4 },
            78  => { self.addr_absolute(mem);          self.inst_lsr(mem);   6 },
            79  => { self.addr_absolute(mem);          self.inst_sre(mem);   6 },
            80  => { self.addr_relative_branch(mem);   self.inst_bvc(mem);   2 },
            81  => { self.addr_indirect_y(mem);        self.inst_eor(mem);   5 },
            82  => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            83  => { self.addr_indirect_y(mem);        self.inst_sre(mem);   8 },
            84  => { self.addr_zeropage_x(mem);        self.inst_nop(mem);   4 },
            85  => { self.addr_zeropage_x(mem);        self.inst_eor(mem);   4 },
            86  => { self.addr_zeropage_x(mem);        self.inst_lsr(mem);   6 },
            87  => { self.addr_zeropage_x(mem);        self.inst_sre(mem);   6 },
            88  => { self.addr_implied(mem);           self.inst_cli(mem);   2 },
            89  => { self.addr_absolute_y(mem);        self.inst_eor(mem);   4 },
            90  => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            91  => { self.addr_absolute_y(mem);        self.inst_sre(mem);   7 },
            92  => { self.addr_absolute_x(mem);        self.inst_nop(mem);   4 },
            93  => { self.addr_absolute_x(mem);        self.inst_eor(mem);   4 },
            94  => { self.addr_absolute_x(mem);        self.inst_lsr(mem);   7 },
            95  => { self.addr_absolute_x(mem);        self.inst_sre(mem);   7 },
            96  => { self.addr_implied(mem);           self.inst_rts(mem);   6 },
            97  => { self.addr_indirect_x(mem);        self.inst_adc(mem);   6 },
            98  => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            99  => { self.addr_indirect_x(mem);        self.inst_rra(mem);   8 },
            100 => { self.addr_zeropage(mem);          self.inst_nop(mem);   3 },
            101 => { self.addr_zeropage(mem);          self.inst_adc(mem);   3 },
            102 => { self.addr_zeropage(mem);          self.inst_ror(mem);   5 },
            103 => { self.addr_zeropage(mem);          self.inst_rra(mem);   5 },
            104 => { self.addr_implied(mem);           self.inst_pla(mem);   4 },
            105 => { self.addr_immediate(mem);         self.inst_adc(mem);   2 },
            106 => { self.addr_accumulator(mem);       self.inst_ror(mem);   2 },
            107 => { self.addr_immediate(mem);         self.inst_nop(mem);   2 },
            108 => { self.addr_indirect(mem);          self.inst_jmp(mem);   5 },
            109 => { self.addr_absolute(mem);          self.inst_adc(mem);   4 },
            110 => { self.addr_absolute(mem);          self.inst_ror(mem);   6 },
            111 => { self.addr_absolute(mem);          self.inst_rra(mem);   6 },
            112 => { self.addr_relative_branch(mem);   self.inst_bvs(mem);   2 },
            113 => { self.addr_indirect_y(mem);        self.inst_adc(mem);   5 },
            114 => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            115 => { self.addr_indirect_y(mem);        self.inst_rra(mem);   8 },
            116 => { self.addr_zeropage_x(mem);        self.inst_nop(mem);   4 },
            117 => { self.addr_zeropage_x(mem);        self.inst_adc(mem);   4 },
            118 => { self.addr_zeropage_x(mem);        self.inst_ror(mem);   6 },
            119 => { self.addr_zeropage_x(mem);        self.inst_rra(mem);   6 },
            120 => { self.addr_implied(mem);           self.inst_sei(mem);   2 },
            121 => { self.addr_absolute_y(mem);        self.inst_adc(mem);   4 },
            122 => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            123 => { self.addr_absolute_y(mem);        self.inst_rra(mem);   7 },
            124 => { self.addr_absolute_x(mem);        self.inst_nop(mem);   4 },
            125 => { self.addr_absolute_x(mem);        self.inst_adc(mem);   4 },
            126 => { self.addr_absolute_x(mem);        self.inst_ror(mem);   7 },
            127 => { self.addr_absolute_x(mem);        self.inst_rra(mem);   7 },
            128 => { self.addr_immediate(mem);         self.inst_nop(mem);   2 },
            129 => { self.addr_indirect_x(mem);        self.inst_sta(mem);   6 },
            130 => { self.addr_immediate(mem);         self.inst_nop(mem);   2 },
            131 => { self.addr_indirect_x(mem);        self.inst_sax(mem);   6 },
            132 => { self.addr_zeropage(mem);          self.inst_sty(mem);   3 },
            133 => { self.addr_zeropage(mem);          self.inst_sta(mem);   3 },
            134 => { self.addr_zeropage(mem);          self.inst_stx(mem);   3 },
            135 => { self.addr_zeropage(mem);          self.inst_sax(mem);   3 },
            136 => { self.addr_implied(mem);           self.inst_dey(mem);   2 },
            137 => { self.addr_immediate(mem);         self.inst_nop(mem);   2 },
            138 => { self.addr_implied(mem);           self.inst_txa(mem);   2 },
            139 => { self.addr_immediate(mem);         self.inst_nop(mem);   2 },
            140 => { self.addr_absolute(mem);          self.inst_sty(mem);   4 },
            141 => { self.addr_absolute(mem);          self.inst_sta(mem);   4 },
            142 => { self.addr_absolute(mem);          self.inst_stx(mem);   4 },
            143 => { self.addr_absolute(mem);          self.inst_sax(mem);   4 },
            144 => { self.addr_relative_branch(mem);   self.inst_bcc(mem);   2 },
            145 => { self.addr_indirect_y(mem);        self.inst_sta(mem);   6 },
            146 => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            147 => { self.addr_indirect_y(mem);        self.inst_nop(mem);   6 },
            148 => { self.addr_zeropage_x(mem);        self.inst_sty(mem);   4 },
            149 => { self.addr_zeropage_x(mem);        self.inst_sta(mem);   4 },
            150 => { self.addr_zeropage_y(mem);        self.inst_stx(mem);   4 },
            151 => { self.addr_zeropage_y(mem);        self.inst_sax(mem);   4 },
            152 => { self.addr_implied(mem);           self.inst_tya(mem);   2 },
            153 => { self.addr_absolute_y(mem);        self.inst_sta(mem);   5 },
            154 => { self.addr_implied(mem);           self.inst_txs(mem);   2 },
            155 => { self.addr_absolute_y(mem);        self.inst_nop(mem);   5 },
            156 => { self.addr_absolute_x(mem);        self.inst_nop(mem);   5 },
            157 => { self.addr_absolute_x(mem);        self.inst_sta(mem);   5 },
            158 => { self.addr_absolute_y(mem);        self.inst_nop(mem);   5 },
            159 => { self.addr_absolute_y(mem);        self.inst_nop(mem);   5 },
            160 => { self.addr_immediate(mem);         self.inst_ldy(mem);   2 },
            161 => { self.addr_indirect_x(mem);        self.inst_lda(mem);   6 },
            162 => { self.addr_immediate(mem);         self.inst_ldx(mem);   2 },
            163 => { self.addr_indirect_x(mem);        self.inst_lax(mem);   6 },
            164 => { self.addr_zeropage(mem);          self.inst_ldy(mem);   3 },
            165 => { self.addr_zeropage(mem);          self.inst_lda(mem);   3 },
            166 => { self.addr_zeropage(mem);          self.inst_ldx(mem);   3 },
            167 => { self.addr_zeropage(mem);          self.inst_lax(mem);   3 },
            168 => { self.addr_implied(mem);           self.inst_tay(mem);   2 },
            169 => { self.addr_immediate(mem);         self.inst_lda(mem);   2 },
            170 => { self.addr_implied(mem);           self.inst_tax(mem);   2 },
            171 => { self.addr_immediate(mem);         self.inst_nop(mem);   2 },
            172 => { self.addr_absolute(mem);          self.inst_ldy(mem);   4 },
            173 => { self.addr_absolute(mem);          self.inst_lda(mem);   4 },
            174 => { self.addr_absolute(mem);          self.inst_ldx(mem);   4 },
            175 => { self.addr_absolute(mem);          self.inst_lax(mem);   4 },
            176 => { self.addr_relative_branch(mem);   self.inst_bcs(mem);   2 },
            177 => { self.addr_indirect_y(mem);        self.inst_lda(mem);   5 },
            178 => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            179 => { self.addr_indirect_y(mem);        self.inst_lax(mem);   5 },
            180 => { self.addr_zeropage_x(mem);        self.inst_ldy(mem);   4 },
            181 => { self.addr_zeropage_x(mem);        self.inst_lda(mem);   4 },
            182 => { self.addr_zeropage_y(mem);        self.inst_ldx(mem);   4 },
            183 => { self.addr_zeropage_y(mem);        self.inst_lax(mem);   4 },
            184 => { self.addr_implied(mem);           self.inst_clv(mem);   2 },
            185 => { self.addr_absolute_y(mem);        self.inst_lda(mem);   4 },
            186 => { self.addr_implied(mem);           self.inst_tsx(mem);   2 },
            187 => { self.addr_absolute_y(mem);        self.inst_lax(mem);   4 },
            188 => { self.addr_absolute_x(mem);        self.inst_ldy(mem);   4 },
            189 => { self.addr_absolute_x(mem);        self.inst_lda(mem);   4 },
            190 => { self.addr_absolute_y(mem);        self.inst_ldx(mem);   4 },
            191 => { self.addr_absolute_y(mem);        self.inst_lax(mem);   4 },
            192 => { self.addr_immediate(mem);         self.inst_cpy(mem);   2 },
            193 => { self.addr_indirect_x(mem);        self.inst_cmp(mem);   6 },
            194 => { self.addr_immediate(mem);         self.inst_nop(mem);   2 },
            195 => { self.addr_indirect_x(mem);        self.inst_dcp(mem);   8 },
            196 => { self.addr_zeropage(mem);          self.inst_cpy(mem);   3 },
            197 => { self.addr_zeropage(mem);          self.inst_cmp(mem);   3 },
            198 => { self.addr_zeropage(mem);          self.inst_dec(mem);   5 },
            199 => { self.addr_zeropage(mem);          self.inst_dcp(mem);   5 },
            200 => { self.addr_implied(mem);           self.inst_iny(mem);   2 },
            201 => { self.addr_immediate(mem);         self.inst_cmp(mem);   2 },
            202 => { self.addr_implied(mem);           self.inst_dex(mem);   2 },
            203 => { self.addr_immediate(mem);         self.inst_nop(mem);   2 },
            204 => { self.addr_absolute(mem);          self.inst_cpy(mem);   4 },
            205 => { self.addr_absolute(mem);          self.inst_cmp(mem);   4 },
            206 => { self.addr_absolute(mem);          self.inst_dec(mem);   6 },
            207 => { self.addr_absolute(mem);          self.inst_dcp(mem);   6 },
            208 => { self.addr_relative_branch(mem);   self.inst_bne(mem);   2 },
            209 => { self.addr_indirect_y(mem);        self.inst_cmp(mem);   5 },
            210 => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            211 => { self.addr_indirect_y(mem);        self.inst_dcp(mem);   8 },
            212 => { self.addr_zeropage_x(mem);        self.inst_nop(mem);   4 },
            213 => { self.addr_zeropage_x(mem);        self.inst_cmp(mem);   4 },
            214 => { self.addr_zeropage_x(mem);        self.inst_dec(mem);   6 },
            215 => { self.addr_zeropage_x(mem);        self.inst_dcp(mem);   6 },
            216 => { self.addr_implied(mem);           self.inst_cld(mem);   2 },
            217 => { self.addr_absolute_y(mem);        self.inst_cmp(mem);   4 },
            218 => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            219 => { self.addr_absolute_y(mem);        self.inst_dcp(mem);   7 },
            220 => { self.addr_absolute_x(mem);        self.inst_nop(mem);   4 },
            221 => { self.addr_absolute_x(mem);        self.inst_cmp(mem);   4 },
            222 => { self.addr_absolute_x(mem);        self.inst_dec(mem);   7 },
            223 => { self.addr_absolute_x(mem);        self.inst_dcp(mem);   7 },
            224 => { self.addr_immediate(mem);         self.inst_cpx(mem);   2 },
            225 => { self.addr_indirect_x(mem);        self.inst_sbc(mem);   6 },
            226 => { self.addr_immediate(mem);         self.inst_nop(mem);   2 },
            227 => { self.addr_indirect_x(mem);        self.inst_isb(mem);   8 },
            228 => { self.addr_zeropage(mem);          self.inst_cpx(mem);   3 },
            229 => { self.addr_zeropage(mem);          self.inst_sbc(mem);   3 },
            230 => { self.addr_zeropage(mem);          self.inst_inc(mem);   5 },
            231 => { self.addr_zeropage(mem);          self.inst_isb(mem);   5 },
            232 => { self.addr_implied(mem);           self.inst_inx(mem);   2 },
            233 => { self.addr_immediate(mem);         self.inst_sbc(mem);   2 },
            234 => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            235 => { self.addr_immediate(mem);         self.inst_sbc(mem);   2 },
            236 => { self.addr_absolute(mem);          self.inst_cpx(mem);   4 },
            237 => { self.addr_absolute(mem);          self.inst_sbc(mem);   4 },
            238 => { self.addr_absolute(mem);          self.inst_inc(mem);   6 },
            239 => { self.addr_absolute(mem);          self.inst_isb(mem);   6 },
            240 => { self.addr_relative_branch(mem);   self.inst_beq(mem);   2 },
            241 => { self.addr_indirect_y(mem);        self.inst_sbc(mem);   5 },
            242 => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            243 => { self.addr_indirect_y(mem);        self.inst_isb(mem);   8 },
            244 => { self.addr_zeropage_x(mem);        self.inst_nop(mem);   4 },
            245 => { self.addr_zeropage_x(mem);        self.inst_sbc(mem);   4 },
            246 => { self.addr_zeropage_x(mem);        self.inst_inc(mem);   6 },
            247 => { self.addr_zeropage_x(mem);        self.inst_isb(mem);   6 },
            248 => { self.addr_implied(mem);           self.inst_sed(mem);   2 },
            249 => { self.addr_absolute_y(mem);        self.inst_sbc(mem);   4 },
            250 => { self.addr_implied(mem);           self.inst_nop(mem);   2 },
            251 => { self.addr_absolute_y(mem);        self.inst_isb(mem);   7 },
            252 => { self.addr_absolute_x(mem);        self.inst_nop(mem);   4 },
            253 => { self.addr_absolute_x(mem);        self.inst_sbc(mem);   4 },
            254 => { self.addr_absolute_x(mem);        self.inst_inc(mem);   7 },
            255 => { self.addr_absolute_x(mem);        self.inst_isb(mem);   7 },
            _   => { panic!("unimplemented/impossible instruction");           },
        }
    }
}





//static void (*addrtable[256])();
//static void (*optable[256])();


//static void (*addrtable[256])() = {
// /*        |  0  |  1  |  2  |  3  |  4  |  5  |  6  |  7  |  8  |  9  |  A  |  B  |  C  |  D  |  E  |  F  |     */
// /* 0 */     imp, indx,  imp, indx,   zp,   zp,   zp,   zp,  imp,  imm,  acc,  imm, abso, abso, abso, abso, /* 0 */
// /* 1 */     rel, indy,  imp, indy,  zpx,  zpx,  zpx,  zpx,  imp, absy,  imp, absy, absx, absx, absx, absx, /* 1 */
// /* 2 */    abso, indx,  imp, indx,   zp,   zp,   zp,   zp,  imp,  imm,  acc,  imm, abso, abso, abso, abso, /* 2 */
// /* 3 */     rel, indy,  imp, indy,  zpx,  zpx,  zpx,  zpx,  imp, absy,  imp, absy, absx, absx, absx, absx, /* 3 */
// /* 4 */     imp, indx,  imp, indx,   zp,   zp,   zp,   zp,  imp,  imm,  acc,  imm, abso, abso, abso, abso, /* 4 */
// /* 5 */     rel, indy,  imp, indy,  zpx,  zpx,  zpx,  zpx,  imp, absy,  imp, absy, absx, absx, absx, absx, /* 5 */
// /* 6 */     imp, indx,  imp, indx,   zp,   zp,   zp,   zp,  imp,  imm,  acc,  imm,  ind, abso, abso, abso, /* 6 */
// /* 7 */     rel, indy,  imp, indy,  zpx,  zpx,  zpx,  zpx,  imp, absy,  imp, absy, absx, absx, absx, absx, /* 7 */
// /* 8 */     imm, indx,  imm, indx,   zp,   zp,   zp,   zp,  imp,  imm,  imp,  imm, abso, abso, abso, abso, /* 8 */
// /* 9 */     rel, indy,  imp, indy,  zpx,  zpx,  zpy,  zpy,  imp, absy,  imp, absy, absx, absx, absy, absy, /* 9 */
// /* A */     imm, indx,  imm, indx,   zp,   zp,   zp,   zp,  imp,  imm,  imp,  imm, abso, abso, abso, abso, /* A */
// /* B */     rel, indy,  imp, indy,  zpx,  zpx,  zpy,  zpy,  imp, absy,  imp, absy, absx, absx, absy, absy, /* B */
// /* C */     imm, indx,  imm, indx,   zp,   zp,   zp,   zp,  imp,  imm,  imp,  imm, abso, abso, abso, abso, /* C */
// /* D */     rel, indy,  imp, indy,  zpx,  zpx,  zpx,  zpx,  imp, absy,  imp, absy, absx, absx, absx, absx, /* D */
// /* E */     imm, indx,  imm, indx,   zp,   zp,   zp,   zp,  imp,  imm,  imp,  imm, abso, abso, abso, abso, /* E */
// /* F */     rel, indy,  imp, indy,  zpx,  zpx,  zpx,  zpx,  imp, absy,  imp, absy, absx, absx, absx, absx  /* F */
// //};

//static void (*optable[256])() = {
// /*        |  0  |  1  |  2  |  3  |  4  |  5  |  6  |  7  |  8  |  9  |  A  |  B  |  C  |  D  |  E  |  F  |      */
// /* 0 */      brk,  ora,  nop,  slo,  nop,  ora,  asl,  slo,  php,  ora,  asl,  nop,  nop,  ora,  asl,  slo, /* 0 */
// /* 1 */      bpl,  ora,  nop,  slo,  nop,  ora,  asl,  slo,  clc,  ora,  nop,  slo,  nop,  ora,  asl,  slo, /* 1 */
// /* 2 */      jsr,  and,  nop,  rla,  bit,  and,  rol,  rla,  plp,  and,  rol,  nop,  bit,  and,  rol,  rla, /* 2 */
// /* 3 */      bmi,  and,  nop,  rla,  nop,  and,  rol,  rla,  sec,  and,  nop,  rla,  nop,  and,  rol,  rla, /* 3 */
// /* 4 */      rti,  eor,  nop,  sre,  nop,  eor,  lsr,  sre,  pha,  eor,  lsr,  nop,  jmp,  eor,  lsr,  sre, /* 4 */
// /* 5 */      bvc,  eor,  nop,  sre,  nop,  eor,  lsr,  sre,  cli,  eor,  nop,  sre,  nop,  eor,  lsr,  sre, /* 5 */
// /* 6 */      rts,  adc,  nop,  rra,  nop,  adc,  ror,  rra,  pla,  adc,  ror,  nop,  jmp,  adc,  ror,  rra, /* 6 */
// /* 7 */      bvs,  adc,  nop,  rra,  nop,  adc,  ror,  rra,  sei,  adc,  nop,  rra,  nop,  adc,  ror,  rra, /* 7 */
// /* 8 */      nop,  sta,  nop,  sax,  sty,  sta,  stx,  sax,  dey,  nop,  txa,  nop,  sty,  sta,  stx,  sax, /* 8 */
// /* 9 */      bcc,  sta,  nop,  nop,  sty,  sta,  stx,  sax,  tya,  sta,  txs,  nop,  nop,  sta,  nop,  nop, /* 9 */
// /* A */      ldy,  lda,  ldx,  lax,  ldy,  lda,  ldx,  lax,  tay,  lda,  tax,  nop,  ldy,  lda,  ldx,  lax, /* A */
// /* B */      bcs,  lda,  nop,  lax,  ldy,  lda,  ldx,  lax,  clv,  lda,  tsx,  lax,  ldy,  lda,  ldx,  lax, /* B */
// /* C */      cpy,  cmp,  nop,  dcp,  cpy,  cmp,  dec,  dcp,  iny,  cmp,  dex,  nop,  cpy,  cmp,  dec,  dcp, /* C */
// /* D */      bne,  cmp,  nop,  dcp,  nop,  cmp,  dec,  dcp,  cld,  cmp,  nop,  dcp,  nop,  cmp,  dec,  dcp, /* D */
// /* E */      cpx,  sbc,  nop,  isb,  cpx,  sbc,  inc,  isb,  inx,  sbc,  nop,  sbc,  cpx,  sbc,  inc,  isb, /* E */
// /* F */      beq,  sbc,  nop,  isb,  nop,  sbc,  inc,  isb,  sed,  sbc,  nop,  isb,  nop,  sbc,  inc,  isb  /* F */
//};

//static const uint32_t ticktable[256] = {
// /*        |  0  |  1  |  2  |  3  |  4  |  5  |  6  |  7  |  8  |  9  |  A  |  B  |  C  |  D  |  E  |  F  |     */
// /* 0 */      7,    6,    2,    8,    3,    3,    5,    5,    3,    2,    2,    2,    4,    4,    6,    6,  /* 0 */
// /* 1 */      2,    5,    2,    8,    4,    4,    6,    6,    2,    4,    2,    7,    4,    4,    7,    7,  /* 1 */
// /* 2 */      6,    6,    2,    8,    3,    3,    5,    5,    4,    2,    2,    2,    4,    4,    6,    6,  /* 2 */
// /* 3 */      2,    5,    2,    8,    4,    4,    6,    6,    2,    4,    2,    7,    4,    4,    7,    7,  /* 3 */
// /* 4 */      6,    6,    2,    8,    3,    3,    5,    5,    3,    2,    2,    2,    3,    4,    6,    6,  /* 4 */
// /* 5 */      2,    5,    2,    8,    4,    4,    6,    6,    2,    4,    2,    7,    4,    4,    7,    7,  /* 5 */
// /* 6 */      6,    6,    2,    8,    3,    3,    5,    5,    4,    2,    2,    2,    5,    4,    6,    6,  /* 6 */
// /* 7 */      2,    5,    2,    8,    4,    4,    6,    6,    2,    4,    2,    7,    4,    4,    7,    7,  /* 7 */
// /* 8 */      2,    6,    2,    6,    3,    3,    3,    3,    2,    2,    2,    2,    4,    4,    4,    4,  /* 8 */
// /* 9 */      2,    6,    2,    6,    4,    4,    4,    4,    2,    5,    2,    5,    5,    5,    5,    5,  /* 9 */
// /* A */      2,    6,    2,    6,    3,    3,    3,    3,    2,    2,    2,    2,    4,    4,    4,    4,  /* A */
// /* B */      2,    5,    2,    5,    4,    4,    4,    4,    2,    4,    2,    4,    4,    4,    4,    4,  /* B */
// /* C */      2,    6,    2,    8,    3,    3,    5,    5,    2,    2,    2,    2,    4,    4,    6,    6,  /* C */
// /* D */      2,    5,    2,    8,    4,    4,    6,    6,    2,    4,    2,    7,    4,    4,    7,    7,  /* D */
// /* E */      2,    6,    2,    8,    3,    3,    5,    5,    2,    2,    2,    2,    4,    4,    6,    6,  /* E */
// /* F */      2,    5,    2,    8,    4,    4,    6,    6,    2,    4,    2,    7,    4,    4,    7,    7   /* F */
//};



//void step6502() {
//    opcode = read6502(pc++);
//    status |= FLAG_CONSTANT;

//    penaltyop = 0;
//    penaltyaddr = 0;

//    (*addrtable[opcode])();
//    (*optable[opcode])();
//    clockticks6502 += ticktable[opcode];
//    if (penaltyop && penaltyaddr) clockticks6502++;
//    clockgoal6502 = clockticks6502;

//    instructions++;

//    if (callexternal) (*loopexternal)();
//}

//void hookexternal(void *funcptr) {
//    if (funcptr != (void *)NULL) {
//        loopexternal = funcptr;
//        callexternal = 1;
//    } else callexternal = 0;
//}

/* For testing purposes */
// struct DbgMem {
//     mem: [u8; std::u16::MAX as usize + 1],
// }
// impl Backplane for DbgMem {
//     fn read(&self, address: u16) -> u8 {
//         println!("READ: {} (returning {})", address, self.mem[address as usize]);
//         self.mem[address as usize]
//     }
//     fn write(&mut self, address: u16, value: u8) {
//         println!("WRITE: Set address {} = {}", address, value);
//         self.mem[address as usize] = value;
//     }
// }
// fn main() {
//     let mut tpu = CPU::new();
//     let mut dbgm = DbgMem { mem: [0 as u8; std::u16::MAX as usize + 1] };
//     let our_val: u16 = 65535;
//     tpu.push16(&mut dbgm, our_val);
//     assert!(tpu.pull16(&mut dbgm) == our_val);
//     tpu.inst_lda(&mut dbgm);
//     //tpu.exec::<DbgMem, FnMut(&mut CPU) -> ()>(&mut dbgm, 100, None);
// }
