#include "asm_regnames.h"

.set noreorder

.text
.globl __start

__start:
    addi v0, $0, 4
    addi v1, $0, 4
    beq v0, v1, Label1
    addi a0, $0, 4
    j Label3

Label1:
    addi a0, $0, 3
    bne v1, a0, Label2

Label2:
    addi a1, $0, 12
    jr a1

Label3:
    jal final

final:
    addi a2, $0, 42
    break
