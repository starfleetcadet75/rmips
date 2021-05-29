# RMIPS

[![Lines of Code](https://tokei.rs/b1/github/starfleetcadet75/rmips)](https://github.com/starfleetcadet75/rmips)

RMIPS is a MIPS R3000 virtual machine simulator written in Rust and based on [VMIPS](http://www.dgate.org/vmips).

## Install

Check the releases page for precompiled binaries.

To build from source, first install [Rust](https://www.rust-lang.org/learn/get-started) then run the following:

```bash
git clone https://github.com/starfleetcadet75/rmips.git
cd rmips
cargo build --release
```

## Development

Install the `gcc-mips-linux-gnu` package in order to cross-compile for MIPS targets.
Use the [examples](./examples) directory as a starting point for creating ROMs.

## GDB Support

RMIPS exposes a GDB stub that can be used for debugging emulated programs.
Start RMIPS with the `--debug` flag to enable the GDB server:

```bash
$ cargo run ./examples/build/emptymain_le.rom --debug
Interpreting ROM file as Little-Endian
Mapping ROM image (./examples/build/emptymain_le.rom, 223 words) to physical address 0x1fc00000
Mapping RAM module (1024KB) to physical address 0x00000000
Mapping Halt Device to physical address 0x01010024
Mapping Test Device to physical address 0x02010000

*************[ RESET ]*************

Waiting for a GDB connection on "127.0.0.1:9001"...
```

You can then connect to it from another shell with `gdb-multiarch`:

```bash
$ gdb-multiarch emptymain_le.elf
(gdb) target remote 127.0.0.1:9001
(gdb) break begin
(gdb) c
```

*Note:* The ROM file does not contain any debugging information.
Use the ELF program with GDB so that it can show source information.

## References

* [VMIPS](http://www.dgate.org/vmips)
* [MIPS R3000 Manual](https://cgi.cse.unsw.edu.au/~cs3231/doc/R3000.pdf)
* [MIPS32 Architecture For Programmers Volume I: Introduction to the MIPS32 Architecture](http://www.cs.cornell.edu/courses/cs3410/2008fa/MIPS_Vol1.pdf)
* [MIPS32 Architecture For Programmers Volume II: The MIPS32 Instruction Set](http://www.cs.cornell.edu/courses/cs3410/2008fa/MIPS_Vol2.pdf)
* [MIPS32 Architecture For Programmers Volume III: The MIPS32 Privileged Resource Architecture](https://www.cs.cornell.edu/courses/cs3410/2008fa/MIPS_Vol3.pdf)
* [Software Solutions for Single Instruction Issue, in Order Processors](https://web.ics.purdue.edu/~vaneet/Aggarwal2004_1425.pdf)
* [GNU AS MIPS Dependent Features](https://sourceware.org/binutils/docs-2.26/as/MIPS_002dDependent.html#MIPS_002dDependent)
* [Detecting MIPS Emulation](https://www.alchemistowl.org/pocorgtfo/pocorgtfo06.pdf)

## License

See the [LICENSE file](LICENSE.md).
