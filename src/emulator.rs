use crate::cpu::cpu::CPU;
use crate::devices::halt_device::{HaltDevice, HALTDEV_BASE_ADDRESS};
use crate::devices::test_device::{TestDevice, TESTDEV_BASE_ADDRESS};
use crate::memory::mapper::Mapper;
use crate::memory::ram::RAM;
use crate::memory::range::Range;
use crate::memory::rom::ROM;
use crate::util::constants::KSEG1_CONST_TRANSLATION;
use crate::util::error::RmipsError;
use crate::util::opts::Opts;
use gdbstub::{DisconnectReason, GdbStub, GdbStubError};
use std::net::{TcpListener, TcpStream};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EmulationEvent {
    Step,
    Halted,
    Breakpoint,
}

pub struct Emulator {
    pub cpu: CPU,
    pub(crate) memory: Mapper,
    pub(crate) breakpoints: Vec<u32>,
    opts: Opts,
    instruction_count: usize,
}

impl Emulator {
    pub fn new(opts: Opts) -> Result<Self, RmipsError> {
        // Setup the different machine components
        // let intc = IntCtrl::new();
        let mut memory = Mapper::new();

        // Setup and connect devices
        Self::setup_rom(&opts, &mut memory)?;
        Self::setup_ram(&opts, &mut memory)?;
        Self::setup_haltdevice(&opts, &mut memory)?;
        // Self::setup_clock()?;
        Self::setup_testdevice(&mut memory)?;

        let mut cpu = CPU::new(opts.instrdump);
        cpu.reset();

        Ok(Emulator {
            cpu,
            memory,
            breakpoints: Vec::new(),
            opts,
            instruction_count: 0,
        })
    }

    pub fn run(&mut self) -> Result<(), RmipsError> {
        if self.opts.debug {
            let connection = Self::wait_for_tcp(9001)?;
            let mut debugger = GdbStub::new(connection);

            match debugger.run(self) {
                Ok(disconnect_reason) => match disconnect_reason {
                    DisconnectReason::Disconnect => {
                        // Run the program to completion
                        self.run_until_halt()?;
                    }
                    DisconnectReason::TargetHalted => eprintln!("Target halted"),
                    DisconnectReason::Kill => eprintln!("GDB sent a kill command"),
                },
                Err(GdbStubError::TargetError(e)) => {
                    eprintln!("GDB target raised a fatal error: {:?}", e);
                }
                Err(_) => todo!("Cover other errors"), // return Err(e.into()),
            }
        } else {
            self.run_until_halt()?;
        }

        Ok(())
    }

    // Steps the CPU state until a halt event is triggered
    fn run_until_halt(&mut self) -> Result<(), RmipsError> {
        loop {
            if self.step()? == EmulationEvent::Halted {
                // Dumps the CPU registers and stack on program halt
                if self.opts.haltdumpcpu {
                    eprintln!("*************[ CPU State ]*************");
                    eprintln!("{}", self.cpu);
                }

                // Dumps the CP0 registers and the contents of the TLB on program halt
                if self.opts.haltdumpcp0 {
                    eprintln!("*************[ CP0 State ]*************");
                    eprintln!("{}", self.cpu.cpzero);
                }

                eprintln!("\n*************[ HALT ]*************\n");
                break;
            }
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<EmulationEvent, RmipsError> {
        self.cpu.step(&mut self.memory)?;
        self.instruction_count += 1;

        // Dump register states after each instruction if requested
        if self.opts.dumpcpu {
            eprintln!("*************[ CPU State ]*************");
            eprintln!("{}", self.cpu);
        }

        if self.breakpoints.contains(&self.cpu.pc) {
            Ok(EmulationEvent::Breakpoint)
        } else if self.instruction_count == 40 {
            Ok(EmulationEvent::Halted)
        } else {
            Ok(EmulationEvent::Step)
        }
    }

    fn setup_rom(opts: &Opts, physmem: &mut Mapper) -> Result<(), RmipsError> {
        // Translate the provided virtual load address to a physical address
        let loadaddress = opts.loadaddress;
        if loadaddress < KSEG1_CONST_TRANSLATION {
            panic!("Provided load address must be greater than 0xa0000000");
        }
        let paddress = loadaddress - KSEG1_CONST_TRANSLATION;

        // Load the provided ROM file
        let rom_path = &opts.romfile;
        let rom = ROM::new(rom_path);

        eprintln!(
            "Mapping ROM image ({}, {} words) to physical address 0x{:08x}",
            rom_path,
            rom.get_extent() / 4,
            paddress
        );
        physmem.map_at_physical_address(Box::new(rom), paddress)
    }

    // Create a new RAM module to install at physical address zero
    fn setup_ram(opts: &Opts, physmem: &mut Mapper) -> Result<(), RmipsError> {
        let ram = RAM::new(opts.memsize);
        let paddress = 0;

        eprintln!(
            "Mapping RAM module ({}KB) to physical address 0x{:08x}",
            ram.get_extent() / 1024,
            paddress
        );
        physmem.map_at_physical_address(Box::new(ram), paddress)
    }

    fn setup_haltdevice(opts: &Opts, physmem: &mut Mapper) -> Result<(), RmipsError> {
        if !opts.nohaltdevice {
            let halt_device = HaltDevice::new();
            let paddress = HALTDEV_BASE_ADDRESS;
            eprintln!("Mapping Halt Device to physical address 0x{:08x}", paddress);
            physmem.map_at_physical_address(Box::new(halt_device), paddress)?;
        }
        Ok(())
    }

    fn setup_testdevice(physmem: &mut Mapper) -> Result<(), RmipsError> {
        let testdev = TestDevice::new();
        let paddress = TESTDEV_BASE_ADDRESS;
        eprintln!("Mapping Test Device to physical address 0x{:08x}", paddress);
        physmem.map_at_physical_address(Box::new(testdev), paddress)
    }

    fn wait_for_tcp(port: u16) -> Result<TcpStream, RmipsError> {
        let sockaddr = format!("127.0.0.1:{}", port);
        eprintln!("Waiting for a GDB connection on {:?}...", sockaddr);

        let sock = TcpListener::bind(sockaddr)?;
        let (stream, address) = sock.accept()?;
        eprintln!("Debugger connected from {}", address);

        Ok(stream)
    }
}
