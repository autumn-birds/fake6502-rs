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

//use std::num::Wrapping;

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

struct CPU {
    /* 6502 CPU registers: */
    pc: u16,
    sp: u8,
    a: u8,
    x: u8,
    y: u8,
    status: u8,

    /* Helper variables: */
    instructions_ran: u32,
    clockticks: u32,
    clockgoal: u32,
    // This variable is here because some C code tests whether the current addressing mode is 'acc'
    // (accumulator) by checking against the function lookup tables the C version of this emulator
    // used. Since I opted to turn those into a match in the Rust version, I needed a different way
    // to implement the same conditional.
    addr_acc: bool,
    // Some of these are probably not needed in the struct, but for a direct port, it's easiest,
    // perhaps, to start out not trying to reason about whether a variable's state being carried
    // over between calls will matter.
    oldpc: u16,
    // EA = Effective Address? This one *is* needed: Addressing modes are implemented as functions
    // that read a byte and set this value to the appropriate address.
    ea: u16,
    reladdr: u16,
    //uint8_t penaltyop, penaltyaddr;
    penaltyop: u8,
    penaltyaddr: u8,
    value: u16,
    result: u16,
    opcode: u8,
    oldstatus: u8,
}

//externally supplied functions
//extern uint8_t read6502(uint16_t address);
//extern void write6502(uint16_t address, uint8_t value);

trait Memory {
    // It might make sense to just use a &mut [u8] for example, but I feel like there's probably a
    // reason the original code did it this way: Any special behavior or mappings for special
    // memory addresses you want to have in the callback function, you can have.  (Consider e.g.
    // real systems where writing to a particular address actually controlled hardware.)
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}

impl CPU {
    // You're going to notice that nearly all of these functions take a 'mem' value that must imply
    // trait Memory as an argument argument alongside the CPU struct.
    //
    // This may prove to have been a mistake, or may not. I felt that it would allow for more
    // flexibility on the caller's part and perhaps less borrowing tangles if we do not require the
    // CPU struct to own its memory.

    fn new() -> CPU {
        CPU { pc: 0, sp: 0xFD, a: 0, x: 0, y: 0, status: 0, addr_acc: false, instructions_ran: 0, clockticks: 0, clockgoal: 0, oldpc: 0, ea: 0, reladdr: 0, penaltyaddr: 0, penaltyop: 0, value: 0, result: 0, opcode: 0, oldstatus: 0 }
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
    fn push16<T: Memory>(&mut self, mem: &mut T, pushval: u16) {
        mem.write(BASE_STACK + (self.sp as u16), ((pushval >> 8) & 0x00FF) as u8);
        self.sp -= 1;
        mem.write(BASE_STACK + (self.sp as u16), (pushval & 0x00FF) as u8);
        self.sp -= 1;
    }

    //void push8(uint8_t pushval) {
    //    write6502(BASE_STACK + sp--, pushval);
    //}
    fn push8<T: Memory>(&mut self, mem: &mut T, pushval: u8) {
        mem.write(BASE_STACK + (self.sp as u16), pushval);
        self.sp -= 1;
    }

    //uint16_t pull16() {
    //    uint16_t temp16;
    //    temp16 = read6502(BASE_STACK + ((sp + 1) & 0xFF)) | ((uint16_t)read6502(BASE_STACK + ((sp + 2) & 0xFF)) << 8);
    //    sp += 2;
    //    return(temp16);
    //}
    fn pull16<T: Memory>(&mut self, mem: &T) -> u16 {
        let mut val: u16 = mem.read(BASE_STACK + ((self.sp as u16 + 1) & 0x00FF)) as u16;
        val            |= (mem.read(BASE_STACK + ((self.sp as u16 + 2) & 0x00FF)) as u16) << 8;
        self.sp += 2;
        val
    }

    //uint8_t pull8() {
    //    return (read6502(BASE_STACK + ++sp));
    //}
    fn pull8<T: Memory>(&mut self, mem: &T) -> u8 {
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
    fn reset<T: Memory>(&mut self, mem: &T) {
        self.pc = mem.read(0xFFFC) as u16 | ((mem.read(0xFFFD) as u16) << 8);
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.status |= FLAG_CONSTANT;
    }


    //addressing mode functions, calculates effective addresses
    //static void imp() { //implied
    //}
    fn addr_implied<T: Memory>(&mut self, _mem: &T) {
    }

    //static void acc() { //accumulator
    //}
    fn addr_accumulator<T: Memory>(&mut self, _mem: &T) {
        self.addr_acc = true;
    }

    //static void imm() { //immediate
    //    ea = pc++;
    //}
    fn addr_immediate<T: Memory>(&mut self, _mem: &T) {
        self.ea = self.pc as u16;
        self.pc += 1;
    }

    //static void zp() { //zero-page
    //    ea = (uint16_t)read6502((uint16_t)pc++);
    //}
    fn addr_zeropage<T: Memory>(&mut self, mem: &T) {
        self.ea = mem.read(self.pc) as u16;
        self.pc += 1;
    }

    //static void zpx() { //zero-page,X
    //    ea = ((uint16_t)read6502((uint16_t)pc++) + (uint16_t)x) & 0xFF; //zero-page wraparound
    //}
    fn addr_zeropage_x<T: Memory>(&mut self, mem: &T) {
        self.ea = mem.read(self.pc) as u16 + (self.x as u16 & 0x00FF);
        // ( the & 0x00FF thing for zero-page wraparound)
        self.pc += 1;
    }

    //static void zpy() { //zero-page,Y
    //    ea = ((uint16_t)read6502((uint16_t)pc++) + (uint16_t)y) & 0xFF; //zero-page wraparound
    //}
    fn addr_zeropage_y<T: Memory>(&mut self, mem: &T) {
        self.ea = mem.read(self.pc) as u16 + (self.y as u16 & 0x00FF);
        // ( the & 0x00FF thing maybe for zero-page wraparound? blehhh)
        self.pc += 1;
    }

    //static void rel() { //relative for branch ops (8-bit immediate value, sign-extended)
    //    reladdr = (uint16_t)read6502(pc++);
    //    if (reladdr & 0x80) reladdr |= 0xFF00;
    //}
    fn addr_relative_branch<T: Memory>(&mut self, mem: &T) {
        self.reladdr = mem.read(self.pc) as u16;
        if self.reladdr & 0x0080 != 0 {
            self.reladdr |= 0xFF00;
        }
        self.pc += 1;
    }

    //static void abso() { //absolute
    //    ea = (uint16_t)read6502(pc) | ((uint16_t)read6502(pc+1) << 8);
    //    pc += 2;
    //}
    fn addr_absolute<T: Memory>(&mut self, mem: &T) {
        self.ea = mem.read(self.pc) as u16;
        self.ea |= (mem.read(self.pc + 1) as u16) << 8;
        self.pc += 2;
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
    fn addr_absolute_x<T: Memory>(&mut self, mem: &T) {
        let startpage: u16;
        self.ea = mem.read(self.pc) as u16 | (mem.read(self.pc + 1) as u16) << 8;
        startpage = self.ea & 0xFF00;
        self.ea += self.x as u16;

        if startpage != (self.ea & 0xFF00) {
            // original source: "one cycle penalty for page-crossing on some opcodes"
            self.penaltyaddr = 1;
        }

        self.pc += 2;
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
    fn addr_absolute_y<T: Memory>(&mut self, mem: &T) {
        let startpage: u16;
        self.ea = mem.read(self.pc) as u16 | (mem.read(self.pc + 1) as u16) << 8;
        startpage = self.ea & 0xFF00;
        self.ea += self.y as u16;

        if startpage != (self.ea & 0xFF00) {
            // original source: "one cycle penalty for page-crossing on some opcodes"
            self.penaltyaddr = 1;
        }

        self.pc += 2;
    }

    //static void ind() { //indirect
    //    uint16_t eahelp, eahelp2;
    //    eahelp = (uint16_t)read6502(pc) | (uint16_t)((uint16_t)read6502(pc+1) << 8);
    //    eahelp2 = (eahelp & 0xFF00) | ((eahelp + 1) & 0x00FF); //replicate 6502 page-boundary wraparound bug
    //    ea = (uint16_t)read6502(eahelp) | ((uint16_t)read6502(eahelp2) << 8);
    //    pc += 2;
    //}
    fn addr_indirect<T: Memory>(&mut self, mem: &T) {
        let eahelp: u16;
        let eahelp2: u16;
        eahelp = mem.read(self.pc) as u16 | (mem.read(self.pc + 1) as u16) << 8;
        // original source: "replicate 6502 page-boundary wraparound bug"
        eahelp2 = (eahelp & 0xFF00) | ((eahelp + 1) & 0x00FF);
        self.ea = mem.read(eahelp) as u16 | (mem.read(eahelp2) as u16) << 8;
        self.pc += 2;
    }

    //static void indx() { // (indirect,X)
    //    uint16_t eahelp;
    //    eahelp = (uint16_t)(((uint16_t)read6502(pc++) + (uint16_t)x) & 0xFF); //zero-page wraparound for table pointer
    //    ea = (uint16_t)read6502(eahelp & 0x00FF) | ((uint16_t)read6502((eahelp+1) & 0x00FF) << 8);
    //}
    fn addr_indirect_x<T: Memory>(&mut self, mem: &T) {
        let eahelp: u16;
        eahelp = (mem.read(self.pc) as u16 + self.x as u16) & 0x00FF; // original: "zero-page wraparound for table"
        self.ea = mem.read(eahelp & 0x00FF) as u16 | (mem.read((eahelp + 1) & 0x00FF) as u16) << 8;
        self.pc += 1;
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
    fn addr_indirect_y<T: Memory>(&mut self, mem: &T) {
        let eahelp: u16 = mem.read(self.pc) as u16;
        self.pc += 1;
        let eahelp2: u16 = (eahelp & 0xFF00) | ((eahelp + 1) & 0x00FF); // original: "zero-page wraparound"
        self.ea = mem.read(eahelp) as u16 | ((mem.read(eahelp2) as u16) << 8);
        let startpage: u16 = self.ea & 0xFF00;
        self.ea += self.y as u16;

        if startpage != (self.ea & 0xFF00) { // original: "one-cycle penalty for page crossing on some opcodes"
            self.penaltyaddr = 1;
        }
    }


    //static uint16_t getvalue() {
    //    if (addrtable[opcode] == acc) return((uint16_t)a);
    //        else return((uint16_t)read6502(ea));
    //}
    fn getvalue<T: Memory>(&mut self, mem: &T) -> u16 {
        // But why is it u16...?
        if self.addr_acc {
            self.a as u16
        } else {
            mem.read(self.ea) as u16
        }
    }

    //static uint16_t getvalue16() {
    //    return((uint16_t)read6502(ea) | ((uint16_t)read6502(ea+1) << 8));
    //}
    fn getvalue_16<T: Memory>(&mut self, mem: &T) -> u16 {
        mem.read(self.ea) as u16 | ((mem.read(self.ea + 1) as u16) << 8)
    }

    //static void putvalue(uint16_t saveval) {
    //    if (addrtable[opcode] == acc) a = (uint8_t)(saveval & 0x00FF);
    //        else write6502(ea, (saveval & 0x00FF));
    //}
    fn putvalue<T: Memory>(&mut self, mem: &mut T, saveval: u16) {
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

    fn flagcalc_overflow(&mut self, n: u16, m: u8, o: u16) {
        // (The ^ is not exponentiation but bitwise XOR. Most people probably know this.)
        if (n ^ (m as u16)) & ((n ^ o) & 0x0080) != 0 {
            self.flagset(FLAG_OVERFLOW);
        } else {
            self.flagclear(FLAG_OVERFLOW);
        }
    }

    //#define saveaccum(n) a = (uint8_t)((n) & 0x00FF)
    fn save_accumulator(&mut self, n: u16) {
        self.a = (n & 0x00FF) as u8;
    }


    //instruction handler functions
    //static void adc() {
    fn inst_adc<T: Memory>(&mut self, mem: &mut T) {
    //    penaltyop = 1;
        self.penaltyop = 1;
    //    value = getvalue();
        self.value = self.getvalue(mem);
    //    result = (uint16_t)a + value + (uint16_t)(status & FLAG_CARRY);
        self.result = self.a as u16 + self.value + (self.status & FLAG_CARRY) as u16;
    //    carrycalc(result);
    //    zerocalc(result);
    //    overflowcalc(result, a, value);
    //    signcalc(result);
        // I copy these into variables because otherwise the borrow checker complains that I'm
        // trying to use a value (e.g., self.result) I've already borrowed (by calling, e.g.,
        // self.flagcalc_carry). It feels like there ought to be a less dumb way to do it, but for
        // now this at least compiles...
        let (r, a, v) = (self.result, self.a, self.value);
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

                if (self.a & 0x0F) > 0x09 {
                    self.a += 0x06;
                }
                if (self.a & 0xF0) > 0x90 {
                    self.a += 0x60;
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
    fn inst_and<T: Memory>(&mut self, mem: &mut T) {
    //    penaltyop = 1;
    //    value = getvalue();
    //    result = (uint16_t)a & value;
        self.penaltyop = 1;
        self.value = self.getvalue(mem);
        self.result = (self.a as u16) & self.value;
       
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
    fn inst_asl<T: Memory>(&mut self, mem: &mut T) {
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
    fn inst_bcc<T: Memory>(&mut self, _mem: &mut T) {
    //    if ((status & FLAG_CARRY) == 0) {
    //        oldpc = pc;
    //        pc += reladdr;
    //        if ((oldpc & 0xFF00) != (pc & 0xFF00)) clockticks6502 += 2; //check if jump crossed a page boundary
    //            else clockticks6502++;
    //    }
        if (self.status & FLAG_CARRY) == 0 {
            self.oldpc = self.pc;
            self.pc += self.reladdr;
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
    fn inst_bcs<T: Memory>(&mut self, _mem: &mut T) {
    //    if ((status & FLAG_CARRY) == FLAG_CARRY) {
    //        oldpc = pc;
    //        pc += reladdr;
    //        if ((oldpc & 0xFF00) != (pc & 0xFF00)) clockticks6502 += 2; //check if jump crossed a page boundary
    //            else clockticks6502++;
    //    }
    //}
        if (self.status & FLAG_CARRY) == FLAG_CARRY {
            self.oldpc = self.pc;
            self.pc += self.reladdr;
            if (self.oldpc & 0xFF00) != (self.pc & 0xFF00) {
                // original: "check if jump crossed a page boundary"
                self.clockticks += 2;
            } else {
                self.clockticks += 1;
            }
        }
    }

    //static void beq() {
    fn inst_beq<T: Memory>(&mut self, _mem: &mut T) {
    //    if ((status & FLAG_ZERO) == FLAG_ZERO) {
    //        oldpc = pc;
    //        pc += reladdr;
    //        if ((oldpc & 0xFF00) != (pc & 0xFF00)) clockticks6502 += 2; //check if jump crossed a page boundary
    //            else clockticks6502++;
    //    }
    //}
        if (self.status & FLAG_ZERO) == FLAG_ZERO {
            self.oldpc = self.pc;
            self.pc += self.reladdr;
            if (self.oldpc & 0xFF00) != (self.pc & 0xFF00) {
                // original: "check if jump crossed a page boundary"
                self.clockticks += 2;
            } else {
                self.clockticks += 1;
            }
        }
    }

    //static void bit() {
    fn inst_bit<T: Memory>(&mut self, mem: &mut T) {
    //    value = getvalue();
    //    result = (uint16_t)a & value;
       
    //    zerocalc(result);
    //    status = (status & 0x3F) | (uint8_t)(value & 0xC0);
    //}
        self.value = self.getvalue(mem);
        self.result = self.a as u16 & self.value;

        let r = self.result;
        self.flagcalc_zero(r);
        self.status = (self.status & 0x3F) | (self.value & 0x00C0) as u8;
    }

    //static void bmi() {
    fn inst_bmi<T: Memory>(&mut self, _mem: &mut T) {
    //    if ((status & FLAG_SIGN) == FLAG_SIGN) {
    //        oldpc = pc;
    //        pc += reladdr;
    //        if ((oldpc & 0xFF00) != (pc & 0xFF00)) clockticks6502 += 2; //check if jump crossed a page boundary
    //            else clockticks6502++;
    //    }
    //}
        if (self.status & FLAG_SIGN) == FLAG_SIGN {
            self.oldpc = self.pc;
            self.pc += self.reladdr;
            if (self.oldpc & 0xFF00) != (self.pc & 0xFF00) {
                // original: "check if jump crossed a page boundary"
                self.clockticks += 2;
            } else {
                self.clockticks += 1;
            }
        }
    }

    //static void bne() {
    fn inst_bne<T: Memory>(&mut self, _mem: &mut T) {
    //    if ((status & FLAG_ZERO) == 0) {
    //        oldpc = pc;
    //        pc += reladdr;
    //        if ((oldpc & 0xFF00) != (pc & 0xFF00)) clockticks6502 += 2; //check if jump crossed a page boundary
    //            else clockticks6502++;
    //    }
    //}
        if (self.status & FLAG_ZERO) == 0 {
            self.oldpc = self.pc;
            self.pc += self.reladdr;
            if (self.oldpc & 0xFF00) != (self.pc & 0xFF00) {
                // original: "check if jump crossed a page boundary"
                self.clockticks += 2;
            } else {
                self.clockticks += 1;
            }
        }
    }

    //static void bpl() {
    fn inst_bpl<T: Memory>(&mut self, _mem: &mut T) {
    //    if ((status & FLAG_SIGN) == 0) {
    //        oldpc = pc;
    //        pc += reladdr;
    //        if ((oldpc & 0xFF00) != (pc & 0xFF00)) clockticks6502 += 2; //check if jump crossed a page boundary
    //            else clockticks6502++;
    //    }
    //}
        if (self.status & FLAG_SIGN) == 0 {
            self.oldpc = self.pc;
            self.pc += self.reladdr;
            if (self.oldpc & 0xFF00) != (self.pc & 0xFF00) {
                // original: "check if jump crossed a page boundary"
                self.clockticks += 2;
            } else {
                self.clockticks += 1;
            }
        }
    }

    //static void brk() {
    fn inst_brk<T: Memory>(&mut self, mem: &mut T) {
    //    pc++;
    //    push16(pc); //push next instruction address onto stack
    //    push8(status | FLAG_BREAK); //push CPU status to stack
    //    setinterrupt(); //set interrupt flag
    //    pc = (uint16_t)read6502(0xFFFE) | ((uint16_t)read6502(0xFFFF) << 8);
    //}
        self.pc += 1;
        let (pc, stat) = (self.pc, self.status);
        self.push16(mem, pc); // original: "push next instruction address onto stack"
        self.push8(mem, stat | FLAG_BREAK); // original: "push CPU status to stack"
        self.flagset(FLAG_INTERRUPT);
        self.pc = mem.read(0xFFFE) as u16 | ((mem.read(0xFFFF) as u16) << 8);
    }

    //static void bvc() {
    fn inst_bvc<T: Memory>(&mut self, _mem: &mut T) {
    //    if ((status & FLAG_OVERFLOW) == 0) {
    //        oldpc = pc;
    //        pc += reladdr;
    //        if ((oldpc & 0xFF00) != (pc & 0xFF00)) clockticks6502 += 2; //check if jump crossed a page boundary
    //            else clockticks6502++;
    //    }
    //}
        if (self.status & FLAG_OVERFLOW) == 0 {
            self.oldpc = self.pc;
            self.pc += self.reladdr;
            if (self.oldpc & 0xFF00) != (self.pc & 0xFF00) {
                // original: "check if jump crossed a page boundary"
                self.clockticks += 2;
            } else {
                self.clockticks += 1;
            }
        }
    }

    //static void bvs() {
    fn inst_bvs<T: Memory>(&mut self, _mem: &mut T) {
    //    if ((status & FLAG_OVERFLOW) == FLAG_OVERFLOW) {
    //        oldpc = pc;
    //        pc += reladdr;
    //        if ((oldpc & 0xFF00) != (pc & 0xFF00)) clockticks6502 += 2; //check if jump crossed a page boundary
    //            else clockticks6502++;
    //    }
    //}
        if (self.status & FLAG_OVERFLOW) == FLAG_OVERFLOW {
            self.oldpc = self.pc;
            self.pc += self.reladdr;
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
    fn inst_clc<T: Memory>(&mut self, _mem: &mut T) {
        self.flagclear(FLAG_CARRY);
    }

    //static void cld() {
    //    cleardecimal();
    //}
    fn inst_cld<T: Memory>(&mut self, _mem: &mut T) {
        self.flagclear(FLAG_DECIMAL);
    }

    //static void cli() {
    //    clearinterrupt();
    //}
    fn inst_cli<T: Memory>(&mut self, _mem: &mut T) {
        self.flagclear(FLAG_INTERRUPT);
    }

    //static void clv() {
    //    clearoverflow();
    //}
    fn inst_clv<T: Memory>(&mut self, _mem: &mut T) {
        self.flagclear(FLAG_OVERFLOW);
    }

    //static void cmp() {
    fn inst_cmp<T: Memory>(&mut self, mem: &mut T) {
    //    penaltyop = 1;
        self.penaltyop = 1;
    //    value = getvalue();
        self.value = self.getvalue(mem);
    //    result = (uint16_t)a - value;
        self.result = self.a as u16 - self.value;
       
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
    fn inst_cpx<T: Memory>(&mut self, mem: &mut T) {
    //    value = getvalue();
        self.value = self.getvalue(mem);
    //    result = (uint16_t)x - value;
        self.result = self.x as u16 - self.value;
       
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
    fn inst_cpy<T: Memory>(&mut self, mem: &mut T) {
    //    value = getvalue();
    //    result = (uint16_t)y - value;
        self.value = self.getvalue(mem);
        self.result = self.y as u16 - self.value;
       
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
    fn inst_dec<T: Memory>(&mut self, mem: &mut T) {
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
    fn inst_dex<T: Memory>(&mut self, _mem: &mut T) {
    //    x--;
        self.x -= 1;
       
    //    zerocalc(x);
    //    signcalc(x);
        let x = self.x as u16;
        self.flagcalc_zero(x);
        self.flagcalc_sign(x);
    //}
    }

    //static void dey() {
    fn inst_dey<T: Memory>(&mut self, _mem: &mut T) {
    //    y--;
        self.y -= 1;
       
    //    zerocalc(y);
    //    signcalc(y);
        let y = self.y as u16;
        self.flagcalc_zero(y);
        self.flagcalc_sign(y);
    //}
    }

    //static void eor() {
    //    penaltyop = 1;
    //    value = getvalue();
    //    result = (uint16_t)a ^ value;
       
    //    zerocalc(result);
    //    signcalc(result);
       
    //    saveaccum(result);
    //}

    //static void inc() {
    //    value = getvalue();
    //    result = value + 1;
       
    //    zerocalc(result);
    //    signcalc(result);
       
    //    putvalue(result);
    //}

    //static void inx() {
    //    x++;
       
    //    zerocalc(x);
    //    signcalc(x);
    //}

    //static void iny() {
    //    y++;
       
    //    zerocalc(y);
    //    signcalc(y);
    //}

    //static void jmp() {
    //    pc = ea;
    //}

    //static void jsr() {
    //    push16(pc - 1);
    //    pc = ea;
    //}

    //static void lda() {
    //    penaltyop = 1;
    //    value = getvalue();
    //    a = (uint8_t)(value & 0x00FF);
       
    //    zerocalc(a);
    //    signcalc(a);
    //}

    //static void ldx() {
    //    penaltyop = 1;
    //    value = getvalue();
    //    x = (uint8_t)(value & 0x00FF);
       
    //    zerocalc(x);
    //    signcalc(x);
    //}

    //static void ldy() {
    //    penaltyop = 1;
    //    value = getvalue();
    //    y = (uint8_t)(value & 0x00FF);
       
    //    zerocalc(y);
    //    signcalc(y);
    //}

    //static void lsr() {
    //    value = getvalue();
    //    result = value >> 1;
       
    //    if (value & 1) setcarry();
    //        else clearcarry();
    //    zerocalc(result);
    //    signcalc(result);
       
    //    putvalue(result);
    //}

    //static void nop() {
    //    switch (opcode) {
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

    //static void ora() {
    //    penaltyop = 1;
    //    value = getvalue();
    //    result = (uint16_t)a | value;
       
    //    zerocalc(result);
    //    signcalc(result);
       
    //    saveaccum(result);
    //}

    //static void pha() {
    //    push8(a);
    //}

    //static void php() {
    //    push8(status | FLAG_BREAK);
    //}

    //static void pla() {
    //    a = pull8();
       
    //    zerocalc(a);
    //    signcalc(a);
    //}

    //static void plp() {
    //    status = pull8() | FLAG_CONSTANT;
    //}

    //static void rol() {
    //    value = getvalue();
    //    result = (value << 1) | (status & FLAG_CARRY);
       
    //    carrycalc(result);
    //    zerocalc(result);
    //    signcalc(result);
       
    //    putvalue(result);
    //}

    //static void ror() {
    //    value = getvalue();
    //    result = (value >> 1) | ((status & FLAG_CARRY) << 7);
       
    //    if (value & 1) setcarry();
    //        else clearcarry();
    //    zerocalc(result);
    //    signcalc(result);
       
    //    putvalue(result);
    //}

    //static void rti() {
    //    status = pull8();
    //    value = pull16();
    //    pc = value;
    //}

    //static void rts() {
    //    value = pull16();
    //    pc = value + 1;
    //}

    //static void sbc() {
    //    penaltyop = 1;
    //    value = getvalue() ^ 0x00FF;
    //    result = (uint16_t)a + value + (uint16_t)(status & FLAG_CARRY);
       
    //    carrycalc(result);
    //    zerocalc(result);
    //    overflowcalc(result, a, value);
    //    signcalc(result);

    //    #ifndef NES_CPU
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
    //    }
    //    #endif
       
    //    saveaccum(result);
    //}

    //static void sec() {
    //    setcarry();
    //}

    //static void sed() {
    //    setdecimal();
    //}

    //static void sei() {
    //    setinterrupt();
    //}

    //static void sta() {
    //    putvalue(a);
    //}

    //static void stx() {
    //    putvalue(x);
    //}

    //static void sty() {
    //    putvalue(y);
    //}

    //static void tax() {
    //    x = a;
       
    //    zerocalc(x);
    //    signcalc(x);
    //}

    //static void tay() {
    //    y = a;
       
    //    zerocalc(y);
    //    signcalc(y);
    //}

    //static void tsx() {
    //    x = sp;
       
    //    zerocalc(x);
    //    signcalc(x);
    //}

    //static void txa() {
    //    a = x;
       
    //    zerocalc(a);
    //    signcalc(a);
    //}

    //static void txs() {
    //    sp = x;
    //}

    //static void tya() {
    //    a = y;
       
    //    zerocalc(a);
    //    signcalc(a);
    //}

    //undocumented instructions
    //#ifdef UNDOCUMENTED
    //    static void lax() {
    //        lda();
    //        ldx();
    //    }

    //    static void sax() {
    //        sta();
    //        stx();
    //        putvalue(a & x);
    //        if (penaltyop && penaltyaddr) clockticks6502--;
    //    }

    //    static void dcp() {
    //        dec();
    //        cmp();
    //        if (penaltyop && penaltyaddr) clockticks6502--;
    //    }

    //    static void isb() {
    //        inc();
    //        sbc();
    //        if (penaltyop && penaltyaddr) clockticks6502--;
    //    }

    //    static void slo() {
    //        asl();
    //        ora();
    //        if (penaltyop && penaltyaddr) clockticks6502--;
    //    }

    //    static void rla() {
    //        rol();
    //        and();
    //        if (penaltyop && penaltyaddr) clockticks6502--;
    //    }

    //    static void sre() {
    //        lsr();
    //        eor();
    //        if (penaltyop && penaltyaddr) clockticks6502--;
    //    }

    //    static void rra() {
    //        ror();
    //        adc();
    //        if (penaltyop && penaltyaddr) clockticks6502--;
    //    }
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
//    clockgoal6502 += tickcount;
   
//    while (clockticks6502 < clockgoal6502) {
//        opcode = read6502(pc++);
//        status |= FLAG_CONSTANT;

//        penaltyop = 0;
//        penaltyaddr = 0;

// TODO: When we get to this part, make certain to set self.addr_acc to false, otherwise there will
// be some strange behavior that takes place.
//        (*addrtable[opcode])();
//        (*optable[opcode])();
//        clockticks6502 += ticktable[opcode];
//        if (penaltyop && penaltyaddr) clockticks6502++;

//        instructions++;

//        if (callexternal) (*loopexternal)();
//    }

//}

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
struct DbgMem {
    mem: [u8; 65535],
}
impl Memory for DbgMem {
    fn read(&self, address: u16) -> u8 {
        println!("READ: {} (returning 0)", address);
        self.mem[address as usize]
    }
    fn write(&mut self, address: u16, value: u8) {
        println!("WRITE: Set address {} = {}", address, value);
        self.mem[address as usize] = value;
    }
}
fn main() {
    let mut tpu = CPU::new();
    let mut dbgm = DbgMem { mem: [0 as u8; 65535] };
    let our_val: u16 = 65535;
    tpu.push16(&mut dbgm, our_val);
    assert!(tpu.pull16(&mut dbgm) == our_val);
}
