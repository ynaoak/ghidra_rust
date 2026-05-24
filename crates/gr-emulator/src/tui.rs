// Terminal UI helpers for the debugger.

pub struct DebugPrompt {
    pub address: u64,
    pub function_name: Option<String>,
    pub step_count: u64,
}

impl DebugPrompt {
    pub fn render(&self) -> String {
        let func = self.function_name.as_deref().unwrap_or("???");
        format!("[{} @ 0x{:x} step#{}]> ", func, self.address, self.step_count)
    }
}

pub fn format_register_table(regs: &[(String, u64)], columns: usize) -> String {
    let mut out = String::new();
    let non_zero: Vec<_> = regs.iter().filter(|(_, v)| *v != 0).collect();
    for (i, (name, val)) in non_zero.iter().enumerate() {
        out.push_str(&format!("{:<6}= 0x{:016x}  ", name, val));
        if (i + 1) % columns == 0 { out.push('\n'); }
    }
    if !out.ends_with('\n') { out.push('\n'); }
    out
}

pub fn format_memory_dump(data: &[u8], base_addr: u64, width: usize) -> String {
    let mut out = String::new();
    for (i, chunk) in data.chunks(width).enumerate() {
        let addr = base_addr + (i * width) as u64;
        out.push_str(&format!("0x{:08x}  ", addr));
        for b in chunk {
            out.push_str(&format!("{:02x} ", b));
        }
        for _ in chunk.len()..width {
            out.push_str("   ");
        }
        out.push_str(" |");
        for &b in chunk {
            if b.is_ascii_graphic() || b == b' ' {
                out.push(b as char);
            } else {
                out.push('.');
            }
        }
        out.push_str("|\n");
    }
    out
}

pub fn format_disasm_line(addr: u64, bytes: &[u8], mnemonic: &str, operands: &str) -> String {
    let hex: String = bytes.iter().take(8).map(|b| format!("{:02x} ", b)).collect();
    format!("0x{:08x}  {:<24} {} {}", addr, hex, mnemonic, operands)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_prompt() {
        let prompt = DebugPrompt {
            address: 0x401000,
            function_name: Some("main".into()),
            step_count: 42,
        };
        let rendered = prompt.render();
        assert!(rendered.contains("main"));
        assert!(rendered.contains("401000"));
        assert!(rendered.contains("42"));
    }

    #[test]
    fn register_table() {
        let regs = vec![
            ("RAX".into(), 0xDEAD), ("RBX".into(), 0),
            ("RCX".into(), 0xBEEF), ("RDX".into(), 0),
        ];
        let table = format_register_table(&regs, 2);
        assert!(table.contains("RAX"));
        assert!(table.contains("RCX"));
        assert!(!table.contains("RBX")); // zero filtered
    }

    #[test]
    fn memory_dump_format() {
        let data = [0x48, 0x89, 0xe5, 0x90];
        let dump = format_memory_dump(&data, 0x1000, 16);
        assert!(dump.contains("0x00001000"));
        assert!(dump.contains("48 89 e5 90"));
    }

    #[test]
    fn disasm_line() {
        let line = format_disasm_line(0x1000, &[0x90], "nop", "");
        assert!(line.contains("0x00001000"));
        assert!(line.contains("90"));
        assert!(line.contains("nop"));
    }
}
