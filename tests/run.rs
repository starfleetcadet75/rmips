use rmips::emulator::Emulator;
use rmips::util::opts::Opts;

// TODO: Addition of more debugging support should make it easier
// to query CPU state and test the value of things like the registers

#[test]
fn run_emptymain_program() {
    let testfile = String::from("./build/emptymain.rom");
    let mut opts = Opts::new(testfile);
    opts.instrdump = true;

    let mut emulator = Emulator::new(opts);
    let result = emulator.run();

    assert_eq!(result.is_ok(), true);
}
