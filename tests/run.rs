use rmips::emulator::Emulator;
use rmips::util::error::RmipsResult;
use rmips::util::opts::Opts;

#[test]
fn run_emptymain_program() -> RmipsResult<()> {
    let testfile = String::from("./tests/build/emptymain.rom");
    let mut opts = Opts::new(testfile);
    opts.instrdump = true;

    let mut emulator = Emulator::new(opts)?;
    let result = emulator.run();

    assert_eq!(result.is_ok(), true);
    assert_eq!(emulator.cpu.pc, 0xbfc00320);
    Ok(())
}
