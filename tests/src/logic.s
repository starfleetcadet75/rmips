#include "asm_regnames.h"

.text
.globl __start

__start:
    ori a0, $0, 0xffff
    ori a1, $0, 0xffbf

    and a0, a0, a1
    andi a1, a0, 0xffbf

    or a2, a0, a1
    xor a2, a2, a0
    nor a2, a2, a1

    lui t0, 0xffff
    ori t0, t0, 65533

    lui t1, 0xffff
    ori t1, t1, 65534

    slt t1, t0, t1
    sltu t1, t0, t1
    slti t3, t0, -1
    sltiu t0, t0, 10

    or a0, a2, t0
end:
    break
