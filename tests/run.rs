use pretty_assertions::assert_eq;

use rmips::emulator::Emulator;
use rmips::util::error::Result;
use rmips::util::opts::Opts;

#[test]
fn run_emptymain_le_program() -> Result<()> {
    let mut opts = Opts::default();
    opts.romfile = String::from("./tests/build/emptymain_le.rom");
    opts.instrdump = true;

    let mut emulator = Emulator::new(opts)?;
    let result = emulator.run();

    assert_eq!(result.is_ok(), true);
    //assert_eq!(emulator.cpu.pc, 0xbfc00298);
    Ok(())
}
