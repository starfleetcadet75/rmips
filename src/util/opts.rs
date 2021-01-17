use clap::{crate_authors, crate_description, crate_version, Clap};

#[derive(Clap)]
#[clap(version = crate_version!(), author = crate_authors!(), about = crate_description!())]
pub struct Opts {
    /// ROM file to be loaded into memory.
    pub romfile: String,
    /// Print verbose logging output.
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: i32,
    /// Virtual address where the ROM will be loaded.
    #[clap(short, long, default_value = "3217031168")]
    pub loadaddress: u32,
    /// Size of the virtual CPU's physical memory in bytes.
    #[clap(short, long, default_value = "1048576")]
    pub memsize: usize,
    /// Enable GDB stub for debugging.
    #[clap(short, long)]
    pub debug: bool,
    /// TCP port for the GDB stub to listen on.
    #[clap(short = 'p', long = "port", default_value = "9001")]
    pub debugport: u16,
    /// IP address for the GDB stub to listen on.
    #[clap(short = 'i', long = "ip", default_value = "127.0.0.1")]
    pub debugip: String,
    /// Interpret the ROM as a big-endian binary.
    #[clap(long)]
    pub bigendian: bool,
    /// Display the memory mappings for the emulator on startup.
    #[clap(long)]
    pub memmap: bool,
    /// Disassemble and print instructions as they are executed.
    #[clap(long)]
    pub instrdump: bool,
    /// Do not map the halt device into physical memory.
    #[clap(long)]
    pub nohaltdevice: bool,
}

impl Default for Opts {
    fn default() -> Self {
        Opts {
            romfile: String::from(""),
            verbose: 0,
            loadaddress: 3217031168,
            memsize: 1048576,
            debug: false,
            debugport: 9001,
            debugip: String::from("127.0.0.1"),
            bigendian: false,
            memmap: false,
            instrdump: false,
            nohaltdevice: false,
        }
    }
}
