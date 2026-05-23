use gr_core::pcode::PcodeOp;
use gr_loader::Memory;

#[derive(Debug, Clone)]
pub struct LiftedInstruction {
    pub address: u64,
    pub length: u32,
    pub mnemonic: String,
    pub ops: Vec<PcodeOp>,
}

impl LiftedInstruction {
    pub fn display_pcode(&self) -> String {
        let mut out = format!("-- 0x{:08x}: {} ({}B)\n", self.address, self.mnemonic, self.length);
        for op in &self.ops {
            out.push_str(&format!("   {}\n", op));
        }
        out
    }
}

impl std::fmt::Display for LiftedInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:08x} [{:>2} ops] {}", self.address, self.ops.len(), self.mnemonic)
    }
}

pub trait PcodeLift: Send + Sync {
    fn lift_instruction(
        &self,
        memory: &Memory,
        address: u64,
    ) -> Result<LiftedInstruction, LiftError>;

    fn lift_range(
        &self,
        memory: &Memory,
        start: u64,
        count: usize,
    ) -> Result<Vec<LiftedInstruction>, LiftError> {
        let mut results = Vec::new();
        let mut addr = start;
        for _ in 0..count {
            match self.lift_instruction(memory, addr) {
                Ok(lifted) => {
                    addr += lifted.length as u64;
                    results.push(lifted);
                }
                Err(_) => break,
            }
        }
        Ok(results)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LiftError {
    #[error("cannot read at 0x{0:x}")]
    UnreadableAddress(u64),
    #[error("decode failed at 0x{address:x}: {reason}")]
    DecodeFailed { address: u64, reason: String },
    #[error("unsupported instruction at 0x{address:x}: {mnemonic}")]
    Unsupported { address: u64, mnemonic: String },
}
