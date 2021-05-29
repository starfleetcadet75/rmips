/* Linker script to create a standalone MIPS ROM */
SECTIONS {
    ENTRY(__start)

    . = 0xbfc00000;  /* ROM start address */
    .text :
    {
        _stext = .;
        *(.text);
        _etext = .;
    }
    . = 0xa0000000 + (0x100000 * 3 / 4);  /* Match setup.s DATA_START */

    _gp = .;
    _copystart = SIZEOF(.text) + ADDR(.text);

    .data : AT (SIZEOF(.text) + ADDR(.text))
    {
        _data = .;
        *(.data) *(.rdata) *(.rodata*) *(.mdebug) *(.reginfo);
        *(.sdata) *(.scommon) *(.lit8);
        _edata = .;
    }
    _copyend = SIZEOF(.data) + SIZEOF(.text) + ADDR(.text);

    .bss : AT (SIZEOF(.data) + SIZEOF(.text) + ADDR(.text))
    {
        _bss = .;
        *(.bss) *(.sbss) *(COMMON);
        _ebss = .;
    }

    _end = .;

    /DISCARD/ :
    {
        *(.MIPS.abiflags);
        *(.MIPS.options);
        *(.options);
        *(.pdr);
        *(.reginfo);
        *(.note.gnu.build-id);
        *(.gnu.attributes);
        *(.comment);
    }
}
