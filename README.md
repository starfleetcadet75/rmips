# RMIPS

[![Lines of Code](https://tokei.rs/b1/github/starfleetcadet75/rmips)](https://github.com/starfleetcadet75/rmips)

RMIPS is a MIPS R3000 virtual machine simulator written in Rust and based on [VMIPS](http://www.dgate.org/vmips).

## Install

Check the releases page for precompiled binaries for your platform.

To build from source, first install [Rust](https://www.rust-lang.org/learn/get-started) then run the following:

```bash
git clone https://github.com/starfleetcadet75/rmips.git
cd rmips
cargo build --release
```

## Development

Install the `gcc-mips-linux-gnu` Ubuntu package in order to target MIPS using the GNU C compiler.
The `tests` directory can be used as a starting point for creating new programs.

## Debugging

RMIPS exposes a GDB stub that can be used for debugging emulated programs over a local TCP connection.
Start RMIPS with the `--debug` flag to enable the GDB server and then connect to it using `gdb-multiarch`.
Use GDB's `file` command to load data from the program.bin file and not program.rom.
The ROM file does not contain the debugging information that GDB requires.

## References

* [VMIPS](http://www.dgate.org/vmips)
* [MIPS R3000 Manual](https://cgi.cse.unsw.edu.au/~cs3231/doc/R3000.pdf)
* [Software Solutions for Single Instruction Issue, in Order Processors](https://web.ics.purdue.edu/~vaneet/Aggarwal2004_1425.pdf)
* [GNU AS MIPS Dependent Features](https://sourceware.org/binutils/docs-2.26/as/MIPS_002dDependent.html#MIPS_002dDependent)

## License

See the [LICENSE file](LICENSE.md).
