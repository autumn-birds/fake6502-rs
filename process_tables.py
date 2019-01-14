import json

# This file takes the data in instruction_tables.json and spits out a segment
# of code that I pasted into fake6502.rs while I was converting it from Rust to
# C. The data comes directly from the fake6502 source code; I used Vim to tear
# out the extraneous syntax and put it into the json file by hand.

with open("instruction_tables.json") as tables:
    itbls = json.load(tables)

def strip(x):
    return x.strip()

ops = list(map(strip, itbls["opcodes"].split(",")))
modes = list(map(strip, itbls["addressing_modes"].split(",")))
ticks = list(map(strip, itbls["ticks"].split(",")))

assert len(ops) == 256
assert len(modes) == 256
assert len(ticks) == 256

# We basically write the code to go in a match statement for each one. Things
# are a little more complicated because not all my functions have the same
# names.

mode_rustfns = {
    'rel': 'addr_relative_branch',
    'acc': 'addr_accumulator',
    'abso': 'addr_absolute',
    'indx': 'addr_indirect_x',
    'indy': 'addr_indirect_y',
    'absy': 'addr_absolute_y',
    'absx': 'addr_absolute_x',
    'imp': 'addr_implied',
    'zpy': 'addr_zeropage_y',
    'imm': 'addr_immediate',
    'ind': 'addr_indirect',
    'indy': 'addr_indirect_y',
    'zp': 'addr_zeropage',
    'zpx': 'addr_zeropage_x'
}

width_ops = max(map(lambda x: len("inst_%s(mem);" % x), ops)) + 2
width_modes = max(map(lambda x: len(mode_rustfns[x] + "(mem);"), modes)) + 2

def padded(targ_width, value):
    return value + ' '*(targ_width-len(value))

for i in range(0, 256):
    op = "inst_%s(mem);" % ops[i]
    mode = mode_rustfns[modes[i]] + "(mem);"
    cycles = ticks[i]
    print("%d => { self.%s self.%s %s }," % (i, padded(width_modes, mode), padded(width_ops, op), cycles))

