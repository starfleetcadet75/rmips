use pretty_assertions::assert_eq;

use rmips::emulator::Emulator;
use rmips::registers::Register;
use rmips::util::error::Result;
use rmips::util::opts::Opts;

#[ignore]
#[test]
fn arithmetic_program() -> Result<()> {
    let mut opts = Opts::default();
    opts.romfile = String::from("./tests/build/arithmetic.rom");
    opts.instrdump = true;

    let mut emulator = Emulator::new(opts)?;
    let result = emulator.run();
    assert!(result.is_ok(), true);

    assert_eq!(emulator.cpu.reg[Register::S0], 0xffffffff);
    assert_eq!(emulator.cpu.reg[Register::S1], 0xfffffff1);

    assert_eq!(emulator.cpu.reg[Register::S2], 4);
    assert_eq!(emulator.cpu.reg[Register::S3], 0xfffffff1);

    assert_eq!(emulator.cpu.reg[Register::S4], 0xfffffffd);
    assert_eq!(emulator.cpu.reg[Register::S5], 0);

    assert_eq!(emulator.cpu.reg[Register::S6], 3);
    assert_eq!(emulator.cpu.reg[Register::S7], 0x33333332);

    Ok(())
}

#[test]
fn bitwise_program() -> Result<()> {
    let mut opts = Opts::default();
    opts.romfile = String::from("./tests/build/bitwise.rom");
    opts.instrdump = true;

    let mut emulator = Emulator::new(opts)?;
    let result = emulator.run();
    assert!(result.is_ok(), true);

    assert_eq!(emulator.cpu.reg[Register::V0], 0xff);
    assert_eq!(emulator.cpu.reg[Register::V1], 0x7f);

    assert_eq!(emulator.cpu.reg[Register::A0], 0xfe000000);
    assert_eq!(emulator.cpu.reg[Register::A1], 0x7f000000);
    assert_eq!(emulator.cpu.reg[Register::A2], 0xff000000);
    assert_eq!(emulator.cpu.reg[Register::A3], 1);

    assert_eq!(emulator.cpu.reg[Register::T1], 0x7f);
    assert_eq!(emulator.cpu.reg[Register::T2], 0xfe000000);
    assert_eq!(emulator.cpu.reg[Register::T3], 0x7f000000);
    assert_eq!(emulator.cpu.reg[Register::T4], 0xff000000);

    Ok(())
}

#[test]
fn branch_program() -> Result<()> {
    let mut opts = Opts::default();
    opts.romfile = String::from("./tests/build/branch.rom");
    opts.instrdump = true;

    let mut emulator = Emulator::new(opts)?;
    let result = emulator.run();
    assert!(result.is_ok(), true);

    assert_eq!(emulator.cpu.reg[Register::A2], 42);
    Ok(())
}

#[test]
fn logic_program() -> Result<()> {
    let mut opts = Opts::default();
    opts.romfile = String::from("./tests/build/logic.rom");
    opts.instrdump = true;

    let mut emulator = Emulator::new(opts)?;
    let result = emulator.run();
    assert!(result.is_ok(), true);

    assert_eq!(emulator.cpu.reg[Register::A0], 0xffff0040);
    assert_eq!(emulator.cpu.reg[Register::A1], 0xffbf);
    assert_eq!(emulator.cpu.reg[Register::A2], 0xffff0040);
    assert_eq!(emulator.cpu.reg[Register::T3], 1);

    Ok(())
}

#[test]
fn memory_program() -> Result<()> {
    let mut opts = Opts::default();
    opts.romfile = String::from("./tests/build/memory.rom");
    opts.instrdump = true;

    let mut emulator = Emulator::new(opts)?;
    let result = emulator.run();
    assert!(result.is_ok(), true);

    assert_eq!(emulator.cpu.reg[Register::A0], 0xf);
    assert_eq!(emulator.cpu.reg[Register::A1], 0x0f0f);
    assert_eq!(emulator.cpu.reg[Register::A2], 0x0f0f);
    assert_eq!(emulator.cpu.reg[Register::A3], 0x0f0f);

    assert_eq!(emulator.cpu.reg[Register::T0], 0xf);
    assert_eq!(emulator.cpu.reg[Register::T1], 0xf);
    assert_eq!(emulator.cpu.reg[Register::T2], 0xfffff0f0);
    assert_eq!(emulator.cpu.reg[Register::T3], 0xf0f0);
    assert_eq!(emulator.cpu.reg[Register::T4], 0xf);
    assert_eq!(emulator.cpu.reg[Register::T5], 0xf);
    assert_eq!(emulator.cpu.reg[Register::T6], 0xfffffff0);
    assert_eq!(emulator.cpu.reg[Register::T7], 0xf0);

    assert_eq!(emulator.cpu.reg[Register::S0], 0xfffffff0);
    assert_eq!(emulator.cpu.reg[Register::S1], 0xf0);

    Ok(())
}
