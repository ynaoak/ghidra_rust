# ghidra-rust

Rust reimplementation of Ghidra's core binary analysis pipeline. CLI-first, library-first — no GUI.

## Build & Test

```bash
cargo build
cargo test
cargo clippy
```

## CLI Usage

```bash
cargo run -- info <binary>
cargo run -- sections <binary>
cargo run -- symbols <binary> [--kind func|data|import|export]
cargo run -- hexdump <binary> <hex-address> [length]
cargo run -- disasm <binary> [--start <hex-addr>] [-n <count>]
cargo run -- registers <binary>
```

## Workspace Structure

| Crate | Purpose |
|-------|---------|
| `gr-core` | Address model, P-code IR (74 opcodes), data types |
| `gr-loader` | ELF/PE/Mach-O loading via goblin |
| `gr-arch` | Architecture trait, x86/x64 (iced-x86), ARM/AArch64 (capstone) |
| `gr-cli` | CLI binary (`ghidra-rust`) |

## Architecture Decisions

- **SpaceId (u32 index)** instead of pointers for address space references — avoids lifetime complexity
- **SmallVec<[VarnodeData; 3]>** for PcodeOp inputs — most ops have 1-3 inputs
- **bitflags** for SpaceFlags, SectionFlags, MemoryFlags
- **goblin** for binary parsing — production-ready, zero-copy capable
- **iced-x86** for x86/x64 disassembly — 250+ MB/s decode speed
- **capstone** for ARM/AArch64 disassembly — multi-arch support
- **Architecture trait** as the extension point for new ISAs (feature-gated)
- Edition 2024, `thiserror` for error types

## Ghidra Reference

The `ghidra/` submodule contains the original NSA Ghidra source for reference. Key files:
- `Ghidra/Features/Decompiler/src/decompile/cpp/opcodes.hh` — P-code opcode definitions
- `Ghidra/Features/Decompiler/src/decompile/cpp/space.hh` — address space model
- `Ghidra/Features/Decompiler/src/decompile/cpp/pcoderaw.hh` — VarnodeData/PcodeOp structs
