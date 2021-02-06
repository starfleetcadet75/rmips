#include "asm_regnames.h"

.text
.globl __start

__start:
    lui	v0, 0xf0f0
    ori v0, v0, 0x0f0f

    sw v0, 0($0)
    sb v0, 4($0)
    sh v0, 8($0)

    lw v1, 0($0)
    lw a0, 4($0)
    lw a1, 8($0)

    lh a2, 0($0)
    lhu a3, 0($0)
    lb t0, 0($0)
    lbu t1, 0($0)

    lh t2, 2($0)
    lhu t3, 2($0)

    lb t4, 1($0)
    lbu t5, 1($0)

    lb t6, 2($0)
    lbu t7, 2($0)

    lb s0, 3($0)
    lbu s1, 3($0)
end:
    break
