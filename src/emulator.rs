use std::net::{TcpListener, TcpStream};

use gdbstub::GdbStub;
use log::{error, info};

use crate::control::cpu::Cpu;
use crate::control::KSEG1;
use crate::devices::halt_device;
use crate::devices::test_device;
use crate::memory::bus::Bus;
use crate::memory::monitor::{AccessKind, Monitor};
use crate::memory::ram::Ram;
use crate::memory::rom::Rom;
use crate::util::error::{Result, RmipsError};
use crate::util::opts::Opts;
use crate::{Address, Endian};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EmulationEvent {
    Step,
    Halted,
    Breakpoint,
    WatchWrite(Address),
    WatchRead(Address),
}

pub struct Emulator {
    pub(crate) cpu: Cpu,
    pub(crate) bus: Bus,
    pub(crate) breakpoints: Vec<Address>,
    pub(crate) watchpoints: Vec<Address>,
    instruction_count: usize,
    opts: Opts,
}

impl Emulator {
    pub fn new(opts: Opts) -> Result<Emulator> {
        let _endian = match opts.bigendian {
            true => {
                println!("Interpreting ROM file as Big-Endian");
                Endian::Big
            }
            false => {
                println!("Interpreting ROM file as Little-Endian");
                Endian::Little
            }
        };

        // Setup the different machine components
        // let intc = IntCtrl::new();
        let mut bus = Bus::new();

        // Setup and connect the various devices
        setup_rom(&opts, &mut bus)?;
        setup_ram(&opts, &mut bus)?;
        setup_haltdevice(&opts, &mut bus)?;
        // setup_clock()?;
        setup_testdevice(&mut bus)?;

        let mut cpu = Cpu::new(opts.instrdump);
        cpu.reset();

        Ok(Self {
            cpu,
            bus,
            breakpoints: Default::default(),
            watchpoints: Default::default(),
            instruction_count: 0,
            opts,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        println!("\n*************[ RESET ]*************\n");

        // Optionally start the GDB server before the program
        if self.opts.debug {
            let connection = wait_for_tcp(&self.opts.debugip, self.opts.debugport)?;
            let mut debugger = GdbStub::new(connection);

            match debugger.run(self) {
                Ok(reason) => {
                    info!("GDB session closed: {:?}", reason);
                }
                Err(err) => {
                    error!("Error occurred in GDB session: {}", err);
                }
            }

            // Resume execution when the GDB session is disconnected
            if let Err(err) = self.run_until_halt() {
                error!("Failed to resume emulation after GDB disconnected: {}", err);
            }
        } else {
            self.run_until_halt()?;
        }

        Ok(())
    }

    // Steps the `Cpu` state until a halt event is triggered.
    fn run_until_halt(&mut self) -> Result<()> {
        loop {
            if self.step()? == EmulationEvent::Halted {
                println!("Executed {} instructions", self.instruction_count);
                println!("\n*************[ HALT ]*************\n");
                break;
            }
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<EmulationEvent> {
        let mut hit_watchpoint = None;

        let mut monitor = Monitor::new(&mut self.bus, &self.watchpoints, |access| {
            hit_watchpoint = Some(access)
        });

        // Step the `Cpu` until a halt is triggered
        match self.cpu.step(&mut monitor) {
            Err(RmipsError::Halt) => return Ok(EmulationEvent::Halted),
            Err(err) => return Err(err),
            _ => {}
        }

        self.instruction_count += 1;

        if let Some(access) = hit_watchpoint {
            // TODO: Do we need to set PC back one instruction here?
            // self.cpu.pc = self.cpu.pc.wrapping_sub(4);

            Ok(match access.kind {
                AccessKind::Read => EmulationEvent::WatchRead(access.address),
                AccessKind::Write => EmulationEvent::WatchWrite(access.address),
            })
        } else if self.breakpoints.contains(&self.cpu.pc) {
            Ok(EmulationEvent::Breakpoint)
        } else {
            Ok(EmulationEvent::Step)
        }
    }
}

fn setup_rom(opts: &Opts, bus: &mut Bus) -> Result<()> {
    // Translate the provided virtual load address to a physical address
    // Initialization code should be located in kseg1 since it is non-cacheable
    let loadaddress = opts.loadaddress;
    if loadaddress < KSEG1 {
        panic!("Provided load address must be greater than 0xa0000000");
    }
    let paddress = loadaddress - KSEG1;

    // Load the provided ROM file
    let rom_path = &opts.romfile;
    let rom = Rom::new(rom_path.to_string())?;
    let size = rom.size();

    println!(
        "Mapping ROM image ({}, {} words) to physical address 0x{:08x}",
        rom_path,
        size / 4,
        paddress
    );

    bus.register(Box::new(rom), paddress, size)
}

// Create a new RAM module to install at physical address zero
fn setup_ram(opts: &Opts, bus: &mut Bus) -> Result<()> {
    let paddress = 0;
    let ram = Ram::new(opts.memsize);

    println!(
        "Mapping RAM module ({}KB) to physical address 0x{:08x}",
        opts.memsize / 1024,
        paddress
    );

    bus.register(Box::new(ram), paddress, opts.memsize)
}

fn setup_haltdevice(opts: &Opts, bus: &mut Bus) -> Result<()> {
    use halt_device::*;

    if !opts.nohaltdevice {
        let paddress = BASE_ADDRESS;
        let haltdev = HaltDevice;

        println!(
            "Mapping Halt Device to physical address 0x{:08x}",
            BASE_ADDRESS
        );
        bus.register(Box::new(haltdev), paddress, std::mem::size_of::<Address>())
    } else {
        Ok(())
    }
}

fn setup_testdevice(bus: &mut Bus) -> Result<()> {
    use test_device::*;

    let paddress = BASE_ADDRESS;
    let testdev = TestDevice::new();

    println!("Mapping Test Device to physical address 0x{:08x}", paddress);
    bus.register(Box::new(testdev), paddress, DATA_LEN)
}

fn wait_for_tcp(ip: &str, port: u16) -> Result<TcpStream> {
    let sockaddr = format!("{}:{}", ip, port);
    let sock = TcpListener::bind(sockaddr.clone())?;
    println!("Waiting for a GDB connection on {:?}...", sockaddr);

    let (stream, address) = sock.accept()?;
    println!("Debugger connected from {}", address);

    Ok(stream)
}
