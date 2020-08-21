# This setup code is based on setup.s from the vmips examples.
# It initializes the TLB for the processor and then calls the
# entry point at the reset vector.

#include "asm_regnames.h"

# These values should match the values given in ld.script
#define MEM_BASE 0xa0000000
#define MEM_SIZE 0x100000
#define DATA_START MEM_BASE + (MEM_SIZE * 3 / 4)
#define INIT_STACK_BASE	DATA_START - 4
#define NTLBENTRIES 64

.text
.globl __start

# First instruction at the initial reset vector
.ent __start
__start:
    j begin
.end __start

# Halt on user tlb exceptions
.org 0x100
    break 0x0

# Halt on exceptions
.org 0x180
    break 0x0

# Boilerplate setup code to initialize the CPU state
.org 0x200
.globl begin
.ent begin

begin:
    # Clear registers
    .set noat
    move $1, $0
    .set at
    move $2, $0
    move $3, $0
    move $4, $0
    move $5, $0
    move $6, $0
    move $7, $0
    move $8, $0
    move $9, $0
    move $10, $0
    move $11, $0
    move $12, $0
    move $13, $0
    move $14, $0
    move $15, $0
    move $16, $0
    move $17, $0
    move $18, $0
    move $19, $0
    move $20, $0
    move $21, $0
    move $22, $0
    move $23, $0
    move $24, $0
    move $25, $0
    move $26, $0
    move $27, $0
    move $28, $0
    move $29, $0
    move $30, $0
    mtc0 zero, Context
    mtc0 zero, BadVAddr
    mtc0 zero, EPC

    # Initialize the TLB
    li t2, NTLBENTRIES    # t2 = TLB entry number
    li t3, 0x00000000     # t3 = (VPN 0x0, ASID 0x00)
    li t4, 0x00000700     # t4 = (PFN 0x0, D V G)
1:
    addiu t2, t2, -1      # Decrement TLB entry number
    sll t1, t2, 8         # Shift entry number into Index field position
    mtc0 t1, Index        # Set Index
    mtc0 t4, EntryLo      # Clear EntryLo
    mtc0 t3, EntryHi      # Set EntryHi
    tlbwi                 # Write TLB[Index] with (EntryHi, EntryLo)
    addiu t3, t3, 0x1000  # Increment VPN
    addiu t4, t4, 0x1000  # Increment PFN
    bnez t2, 1b           # Go back if we're not done yet
    nop
    mtc0 zero, EntryHi    # Clear EntryHi (sets effective ASID=0x0)

    # Initialize the stack and global pointers
    li sp, INIT_STACK_BASE
    la gp, _gp

    # Copy the program data from ROM to RAM
    la t1, _copystart  # t1 = beginning source address for copy
    la t2, _copyend
    addiu t2, t2, 4    # t2 = one word past ending source address
    move t3, gp        # t3 = beginning dest address
1:
    lw t4, 0(t1)       # load t4 from ROM
    sw t4, 0(t3)       # store it in RAM
    addiu t1, t1, 4    # increment both pointers
    addiu t3, t3, 4
    bne t1, t2, 1b     # Continue looping until transfer is complete
    nop

    # Call the entry function in the actual program
    jal entry

    # Stop execution after the entry function returns
    break 0x0
.end begin

