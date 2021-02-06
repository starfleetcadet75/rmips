#include "asm_regnames.h"

.text
.globl __start

__start:
    ori t0, $0, 2
    ori t1, $0, 5

    sub t0, t0, t1
    add t2, t0, t1
    addu t3, t0, t1
    sub t4, t0, t1
    subu t5, t0, t1
    addi t6, t0, 4

    mult t0, t1
    mfhi s0
    mflo s1

    multu t0, t1
    mfhi s2
    mflo s3

    div t0, t1
    mfhi s4
    mflo s5

    divu t0, t1
    mfhi s6
    mflo s7
end:
    break
