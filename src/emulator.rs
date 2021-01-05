use crate::cpu::cpu::Cpu;
use crate::cpu::KSEG1;
use crate::devices::halt_device::{HaltDevice, HALTDEV_BASE_ADDRESS};
use crate::devices::test_device::{TestDevice, TESTDEV_BASE_ADDRESS};
use crate::memory::mapper::Mapper;
use crate::memory::monitor::{AccessKind, Monitor};
use crate::memory::ram::RAM;
use crate::memory::range::Range;
use crate::memory::rom::ROM;
use crate::memory::Endian;
use crate::util::error::RmipsError;
use crate::util::opts::Opts;
use gdbstub::{DisconnectReason, GdbStub, GdbStubError};
use std::net::{TcpListener, TcpStream};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EmulationEvent {
    Step,
    Halted,
    Breakpoint,
    WatchWrite(u32),
    WatchRead(u32),
}

pub struct Emulator {
    pub cpu: Cpu,
    pub(crate) memory: Mapper,
    pub(crate) breakpoints: Vec<u32>,
    pub(crate) watchpoints: Vec<u32>,
    opts: Opts,
    instruction_count: usize,
}

impl Emulator {
    pub fn new(opts: Opts) -> Result<Self, RmipsError> {
        // Setup the different machine components
        // let intc = IntCtrl::new();
        let mut memory = Mapper::new();

        // Setup and connect the various devices
        Self::setup_rom(&opts, &mut memory)?;
        Self::setup_ram(&opts, &mut memory)?;
        Self::setup_haltdevice(&opts, &mut memory)?;
        // Self::setup_clock()?;
        Self::setup_testdevice(&opts, &mut memory)?;

        let mut cpu = Cpu::new(opts.instrdump);
        cpu.reset();

        Ok(Emulator {
            cpu,
            memory,
            breakpoints: Vec::new(),
            watchpoints: Vec::new(),
            opts,
            instruction_count: 0,
        })
    }

    pub fn run(&mut self) -> Result<(), RmipsError> {
        // Optionally display the memory map on startup
        if self.opts.memmap {
            println!("*************[ MEMORY MAP ]*************");
            println!("{}", self.memory);
        }

        println!("\n*************[ RESET ]*************\n");

        if self.opts.debug {
            let connection = Self::wait_for_tcp(&self.opts.debugip, self.opts.debugport)?;
            let mut debugger = GdbStub::new(connection);

            match debugger.run(self) {
                disconnect_reason => match disconnect_reason {
                    Ok(DisconnectReason::Disconnect) => {
                        // Run the program to completion
                        self.run_until_halt()?;
                    }
                    Ok(DisconnectReason::TargetHalted) => println!("Target halted"),
                    Ok(DisconnectReason::Kill) => println!("GDB sent a kill command"),
                    Err(GdbStubError::TargetError(err)) => return Err(err),
                    Err(err) => eprintln!("{}", err),
                },
            }
        } else {
            self.run_until_halt()?;
        }

        Ok(())
    }

    // Steps the `Cpu` state until a halt event is triggered
    fn run_until_halt(&mut self) -> Result<(), RmipsError> {
        loop {
            if self.step()? == EmulationEvent::Halted {
                // Dumps the registers and stack on program halt
                if self.opts.haltdumpcpu {
                    println!("*************[ CPU State ]*************");
                    println!("{}", self.cpu);
                }

                // Dumps the CP0 registers and the contents of the TLB on program halt
                if self.opts.haltdumpcp0 {
                    println!("*************[ CP0 State ]*************");
                    println!("{}", self.cpu.cpzero);
                }

                println!("Executed {} instructions", self.instruction_count);
                println!("\n*************[ HALT ]*************\n");
                break;
            }
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<EmulationEvent, RmipsError> {
        let mut hit_watchpoint = None;

        let mut monitor = Monitor::new(&mut self.memory, &self.watchpoints, |access| {
            hit_watchpoint = Some(access)
        });

        self.cpu.step(&mut monitor)?;
        self.instruction_count += 1;

        // Dump register states after each instruction if requested
        if self.opts.dumpcpu {
            println!("*************[ CPU State ]*************");
            println!("{}", self.cpu);
        }

        if let Some(access) = hit_watchpoint {
            // TODO: Do we need to set PC back one instruction here?
            // self.cpu.pc = self.cpu.pc.wrapping_sub(4);

            Ok(match access.kind {
                AccessKind::Read => EmulationEvent::WatchRead(access.address),
                AccessKind::Write => EmulationEvent::WatchWrite(access.address),
            })
        } else if self.breakpoints.contains(&self.cpu.pc) {
            Ok(EmulationEvent::Breakpoint)
        } else if self.instruction_count == 40 {
            // Trigger a halt at 40 instructions for now
            Ok(EmulationEvent::Halted)
        } else {
            Ok(EmulationEvent::Step)
        }
    }

    fn setup_rom(opts: &Opts, physmem: &mut Mapper) -> Result<(), RmipsError> {
        let endian = match opts.bigendian {
            true => {
                println!("Interpreting ROM file as Big-Endian");
                Endian::Big
            }
            false => {
                println!("Interpreting ROM file as Little-Endian");
                Endian::Little
            }
        };

        // Translate the provided virtual load address to a physical address
        let loadaddress = opts.loadaddress;
        if loadaddress < KSEG1 {
            panic!("Provided load address must be greater than 0xa0000000");
        }
        let paddress = loadaddress - KSEG1;

        // Load the provided ROM file
        let rom_path = &opts.romfile;
        let rom = ROM::new(endian, rom_path, paddress)?;

        println!(
            "Mapping ROM image ({}, {} words) to physical address 0x{:08x}",
            rom_path,
            rom.get_size() / 4,
            paddress
        );
        physmem.add_range(Box::new(rom))
    }

    // Create a new RAM module to install at physical address zero
    fn setup_ram(opts: &Opts, physmem: &mut Mapper) -> Result<(), RmipsError> {
        let endian = match opts.bigendian {
            true => Endian::Big,
            false => Endian::Little,
        };

        let paddress = 0;
        let ram = RAM::new(endian, opts.memsize, paddress);

        println!(
            "Mapping RAM module ({}KB) to physical address 0x{:08x}",
            ram.get_size() / 1024,
            paddress
        );
        physmem.add_range(Box::new(ram))
    }

    fn setup_haltdevice(opts: &Opts, physmem: &mut Mapper) -> Result<(), RmipsError> {
        if !opts.nohaltdevice {
            let endian = match opts.bigendian {
                true => Endian::Big,
                false => Endian::Little,
            };

            let halt_device = HaltDevice::new(endian);
            println!(
                "Mapping Halt Device to physical address 0x{:08x}",
                HALTDEV_BASE_ADDRESS
            );
            physmem.add_range(Box::new(halt_device))?;
        }
        Ok(())
    }

    fn setup_testdevice(opts: &Opts, physmem: &mut Mapper) -> Result<(), RmipsError> {
        let endian = match opts.bigendian {
            true => Endian::Big,
            false => Endian::Little,
        };

        let testdev = TestDevice::new(endian);
        println!(
            "Mapping Test Device to physical address 0x{:08x}",
            TESTDEV_BASE_ADDRESS
        );
        physmem.add_range(Box::new(testdev))
    }

    fn wait_for_tcp(ip: &str, port: u16) -> Result<TcpStream, RmipsError> {
        let sockaddr = format!("{}:{}", ip, port);
        println!("Waiting for a GDB connection on {:?}...", sockaddr);

        let sock = TcpListener::bind(sockaddr)?;
        let (stream, address) = sock.accept()?;
        println!("Debugger connected from {}", address);

        Ok(stream)
    }
}
