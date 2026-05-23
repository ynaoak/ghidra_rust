use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use gr_arch::arch::create_architecture;
use gr_loader::{BinaryLoader, SymbolKind};

#[derive(Parser)]
#[command(name = "ghidra-rust", version, about = "Binary analysis tool powered by ghidra-rust")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display binary file information
    Info {
        /// Path to the binary file
        file: PathBuf,
    },
    /// List sections in the binary
    Sections {
        /// Path to the binary file
        file: PathBuf,
    },
    /// List symbols in the binary
    Symbols {
        /// Path to the binary file
        file: PathBuf,
        /// Filter by symbol kind (func, data, import, export)
        #[arg(short, long)]
        kind: Option<String>,
    },
    /// Disassemble instructions
    Disasm {
        /// Path to the binary file
        file: PathBuf,
        /// Start address (hex). Defaults to entry point
        #[arg(short, long, value_parser = parse_hex)]
        start: Option<u64>,
        /// Number of instructions to disassemble
        #[arg(short = 'n', long, default_value = "32")]
        count: usize,
    },
    /// List registers for the binary's architecture
    Registers {
        /// Path to the binary file
        file: PathBuf,
    },
    /// Hex dump at a given address
    Hexdump {
        /// Path to the binary file
        file: PathBuf,
        /// Start address (hex, e.g. 0x1000)
        #[arg(value_parser = parse_hex)]
        address: u64,
        /// Number of bytes to dump
        #[arg(default_value = "256")]
        length: usize,
    },
}

fn parse_hex(s: &str) -> Result<u64, String> {
    let s = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s);
    u64::from_str_radix(s, 16).map_err(|e| format!("invalid hex address: {}", e))
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Commands::Info { file } => cmd_info(&file),
        Commands::Sections { file } => cmd_sections(&file),
        Commands::Symbols { file, kind } => cmd_symbols(&file, kind.as_deref()),
        Commands::Disasm {
            file,
            start,
            count,
        } => cmd_disasm(&file, start, count),
        Commands::Registers { file } => cmd_registers(&file),
        Commands::Hexdump {
            file,
            address,
            length,
        } => cmd_hexdump(&file, address, length),
    }
}

fn cmd_info(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let info = BinaryLoader::load(path)?;
    println!("File:         {}", path.display());
    println!("Format:       {}", info.format);
    println!("Architecture: {}", info.arch);
    println!("Bits:         {}", info.bits);
    println!("Endian:       {:?}", info.endian);
    println!("Entry Point:  0x{:x}", info.entry_point);
    println!("Sections:     {}", info.sections.len());
    println!("Symbols:      {}", info.symbols.len());
    Ok(())
}

fn cmd_sections(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let info = BinaryLoader::load(path)?;
    println!(
        "{:<30} {:>16} {:>12} Flags",
        "Name", "Address", "Size"
    );
    println!("{}", "-".repeat(70));
    for section in &info.sections {
        let flags = format!(
            "{}{}{}",
            if section.flags.contains(gr_loader::SectionFlags::READ) {
                "r"
            } else {
                "-"
            },
            if section.flags.contains(gr_loader::SectionFlags::WRITE) {
                "w"
            } else {
                "-"
            },
            if section.flags.contains(gr_loader::SectionFlags::EXECUTE) {
                "x"
            } else {
                "-"
            },
        );
        println!(
            "{:<30} {:>16} {:>12} {}",
            section.name,
            format!("0x{:x}", section.address),
            format!("0x{:x}", section.size),
            flags
        );
    }
    Ok(())
}

fn cmd_symbols(path: &Path, kind_filter: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let info = BinaryLoader::load(path)?;
    println!(
        "{:<8} {:>16} {:>8} Name",
        "Kind", "Address", "Size"
    );
    println!("{}", "-".repeat(60));

    let kind_match = kind_filter.map(|k| match k.to_lowercase().as_str() {
        "func" | "function" => Some(SymbolKind::Function),
        "data" => Some(SymbolKind::Data),
        "import" => Some(SymbolKind::Import),
        "export" => Some(SymbolKind::Export),
        _ => None,
    });

    for sym in &info.symbols {
        if let Some(ref filter) = kind_match
            && let Some(expected) = filter
                && sym.kind != *expected {
                    continue;
                }
        println!(
            "{:<8} {:>16} {:>8} {}",
            format!("{}", sym.kind),
            format!("0x{:x}", sym.address),
            sym.size,
            sym.name
        );
    }
    Ok(())
}

fn cmd_hexdump(
    path: &Path,
    address: u64,
    length: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let info = BinaryLoader::load(path)?;
    let mut buf = vec![0u8; length];
    let bytes_read = match info.memory.read_bytes(address, &mut buf) {
        Ok(()) => length,
        Err(_) => {
            let mut count = 0;
            for (i, slot) in buf.iter_mut().enumerate().take(length) {
                match info.memory.read_byte(address + i as u64) {
                    Some(b) => {
                        *slot = b;
                        count = i + 1;
                    }
                    None => break,
                }
            }
            count
        }
    };

    if bytes_read == 0 {
        println!("No data at address 0x{:x}", address);
        return Ok(());
    }

    for row_start in (0..bytes_read).step_by(16) {
        let row_addr = address + row_start as u64;
        print!("  {:08x}  ", row_addr);

        for col in 0..16 {
            let idx = row_start + col;
            if idx < bytes_read {
                print!("{:02x} ", buf[idx]);
            } else {
                print!("   ");
            }
            if col == 7 {
                print!(" ");
            }
        }

        print!(" |");
        for col in 0..16 {
            let idx = row_start + col;
            if idx < bytes_read {
                let c = buf[idx];
                if c.is_ascii_graphic() || c == b' ' {
                    print!("{}", c as char);
                } else {
                    print!(".");
                }
            }
        }
        println!("|");
    }

    Ok(())
}

fn cmd_disasm(path: &Path, start: Option<u64>, count: usize) -> Result<(), Box<dyn std::error::Error>> {
    let info = BinaryLoader::load(path)?;
    let arch = create_architecture(info.arch)?;
    let address = start.unwrap_or(info.entry_point);

    println!(
        "Disassembly of {} ({}) at 0x{:x}:\n",
        path.display(),
        arch.name(),
        address
    );

    let instructions = arch.decode_linear(&info.memory, address, count)?;
    for insn in &instructions {
        println!("{}", insn);
    }

    if instructions.is_empty() {
        println!("  (no instructions decoded at 0x{:x})", address);
    }
    Ok(())
}

fn cmd_registers(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let info = BinaryLoader::load(path)?;
    let arch = create_architecture(info.arch)?;

    println!("Registers for {} ({}):\n", path.display(), arch.name());
    println!(
        "{:<12} {:>6} {:>10} {:>6}  Aliases",
        "Name", "Size", "Offset", "Space"
    );
    println!("{}", "-".repeat(55));

    for reg in arch.registers() {
        let aliases = if reg.aliases.is_empty() {
            String::new()
        } else {
            reg.aliases.join(", ")
        };
        println!(
            "{:<12} {:>4}B  0x{:06x} {:>6}  {}",
            reg.name,
            reg.varnode.size,
            reg.varnode.offset,
            reg.varnode.space.0,
            aliases
        );
    }

    if let Some(sp) = arch.stack_pointer() {
        println!("\nStack pointer: {}", sp.name);
    }

    if let Some(cc) = arch.default_calling_convention() {
        println!("Default calling convention: {}", cc.name);
    }

    Ok(())
}
