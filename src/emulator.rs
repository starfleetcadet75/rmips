use crate::cpu::CPU;
use crate::devices::halt_device::{HaltDevice, HALTDEV_BASE_ADDRESS};
use crate::devices::ram::RAM;
use crate::devices::rom::ROM;
use crate::devices::test_device::{TestDevice, TESTDEV_BASE_ADDRESS};
use crate::mapper::Mapper;
use crate::util::constants::KSEG1_CONST_TRANSLATION;
use crate::util::error::RmipsResult;
use crate::util::opts::Opts;
use crate::util::range::Range;
use capstone::prelude::*;
use error_chain::bail;
use std::rc::Rc;

/// The possible machine states.
#[derive(Debug, PartialEq)]
pub enum MachineState {
    HALT,
    RUN,
    DEBUG,
    INTERACT,
}

pub struct Emulator {
    opts: Rc<Opts>,
    state: MachineState,
    instruction_count: usize,
}

impl Emulator {
    pub fn new(opts: Opts) -> Emulator {
        Emulator {
            opts: Rc::new(opts),
            state: MachineState::HALT,
            instruction_count: 0,
        }
    }

    pub fn run(&mut self) -> RmipsResult<()> {
        // Setup the different machine components
        // let intc = IntCtrl::new();
        let mut physmem = Mapper::new();

        self.setup_rom(&mut physmem)?;
        self.setup_ram(&mut physmem)?;
        self.setup_haltdevice(&mut physmem)?;
        // self.setup_clock()?;
        self.setup_testdevice(&mut physmem)?;

        // Create an instance of Capstone to use as a disassembler if required
        let disassembler = match self.opts.instrdump {
            true => Some(
                Capstone::new()
                    .mips()
                    .mode(arch::mips::ArchMode::Mips32R6)
                    .detail(true)
                    .build()
                    .expect("Capstone failed to initialize"),
            ),
            false => None,
        };

        let mut cpu = CPU::new(physmem, disassembler);

        // signal(SIGQUIT, halt_machine_by_signal);
        println!("Hit Ctrl-\\ to halt machine, Ctrl-_ for a debug prompt.");

        let debug = true; // if self.opts.gdb
        if debug {}

        // Reset the CPU before starting the emulation
        println!("\n*************[ RESET ]*************\n");
        cpu.reset();
        self.state = MachineState::RUN;

        // This is the main emulation loop where instructions are executed
        // by the CPU while the machine is in the running state
        loop {
            match self.state {
                MachineState::HALT => break,
                MachineState::RUN => {
                    cpu.step()?;

                    // Dump register states after each instruction if requested
                    if self.opts.dumpcpu {
                        cpu.dump_regs();
                        // cpu.dump_stack();
                    }

                    self.instruction_count += 1;

                    // Run the first 10 instructions for testing
                    if self.instruction_count == 40 {
                        self.state = MachineState::HALT;
                    }
                }
                MachineState::DEBUG => {
                    // TODO: Next step should be to get debugging working so that we can test arbitrary values
                    // See https://docs.rs/gdbstub/0.2.0/gdbstub/index.html
                    // and https://github.com/daniel5151/clicky for ideas
                    // dbgr.serverloop();
                }
                MachineState::INTERACT => todo!(),
            }
        }

        // Dumps the CPU registers and stack on program halt
        if self.opts.haltdumpcpu {
            cpu.dump_regs();
            // cpu.dump_stack();
        }

        // Dumps the CP0 registers and the contents of the TLB on program halt
        // if self.opts.haltdumpcp0 {
        //     cpu.cpzero_dump_regs_tlb();
        // }

        println!("\n*************[ HALT ]*************\n");
        Ok(())
    }

    fn setup_rom(&mut self, physmem: &mut Mapper) -> RmipsResult<()> {
        // Translate the provided virtual load address to a physical address
        let loadaddress = self.opts.loadaddress;
        if loadaddress < KSEG1_CONST_TRANSLATION {
            bail!("Provided load address must be greater than 0xa0000000");
        }
        let paddress = loadaddress - KSEG1_CONST_TRANSLATION;

        // Load the provided ROM file
        let rom_path = &self.opts.romfile;
        let rom = ROM::new(rom_path)?;

        println!(
            "Mapping ROM image ({}, {} words) to physical address 0x{:08x}",
            rom_path,
            rom.get_extent() / 4,
            paddress
        );
        physmem.map_at_physical_address(Box::new(rom), paddress)
    }

    // Create a new RAM module to install at physical address zero
    fn setup_ram(&mut self, physmem: &mut Mapper) -> RmipsResult<()> {
        let ram = RAM::new(self.opts.memsize);
        let paddress = 0;

        println!(
            "Mapping RAM module ({}KB) to physical address 0x{:08x}",
            ram.get_extent() / 1024,
            paddress
        );
        physmem.map_at_physical_address(Box::new(ram), paddress)
    }

    fn setup_haltdevice(&mut self, physmem: &mut Mapper) -> RmipsResult<()> {
        if !self.opts.nohaltdevice {
            let halt_device = HaltDevice::new();
            let paddress = HALTDEV_BASE_ADDRESS;
            println!("Mapping Halt Device to physical address 0x{:08x}", paddress);
            physmem.map_at_physical_address(Box::new(halt_device), paddress)?;
        }
        Ok(())
    }

    fn setup_testdevice(&mut self, physmem: &mut Mapper) -> RmipsResult<()> {
        let testdev = TestDevice::new();
        let paddress = TESTDEV_BASE_ADDRESS;
        println!("Mapping Test Device to physical address 0x{:08x}", paddress);
        physmem.map_at_physical_address(Box::new(testdev), paddress)
    }

    /// Halt the simulation.
    fn halt(&mut self) {
        self.state = MachineState::HALT;
    }
}
