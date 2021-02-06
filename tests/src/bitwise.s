#include "asm_regnames.h"

.text
.globl __start

__start:
    addi v0, $0, 0x00ff
    srl v1, v0, 1

    sll a0, v1, 25
    srl a1, a0, 1
    sra a2, a0, 1
    addi a3, $0, 1

    addi t0, $0, 25
    srlv t1, v0, a3
    sllv t2, t1, t0
    srlv t3, t2, a3
    srav t4, t2, a3
end:
    break
