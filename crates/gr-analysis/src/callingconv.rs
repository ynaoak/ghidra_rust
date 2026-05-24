use gr_core::address::SpaceId;
use gr_core::pcode::OpCode;
use gr_lift::PcodeLift;
use gr_program::Program;

use crate::analyzer::{AnalysisError, AnalysisResult, Analyzer};

pub struct CallingConventionAnalyzer;

impl Analyzer for CallingConventionAnalyzer {
    fn name(&self) -> &str {
        "Calling Convention"
    }
    fn description(&self) -> &str {
        "Infers calling conventions from register usage patterns"
    }
    fn priority(&self) -> u32 {
        770
    }
    fn analyze(&self, program: &mut Program) -> Result<AnalysisResult, AnalysisError> {
        if !matches!(program.info.arch, gr_loader::Architecture::X86_64) {
            return Ok(AnalysisResult {
                analyzer_name: self.name().into(),
                functions_found: 0,
                references_found: 0,
                instructions_decoded: 0,
            });
        }

        let lifter: Box<dyn PcodeLift> = Box::new(gr_lift::x86::X86Lifter::new_64());
        let mut inferred = 0;

        let func_entries: Vec<u64> = program.listing.functions().map(|f| f.entry_point).collect();

        for entry in func_entries {
            let lifted = match lifter.lift_range(&program.info.memory, entry, 50) {
                Ok(l) => l,
                Err(_) => continue,
            };

            let win_regs = [0x08u64, 0x10, 0x80, 0x88]; // RCX, RDX, R8, R9
            let sysv_regs = [0x38u64, 0x30, 0x10, 0x08]; // RDI, RSI, RDX, RCX

            let mut win_score = 0;
            let mut sysv_score = 0;

            for insn in &lifted {
                for op in &insn.ops {
                    for inp in &op.inputs {
                        if inp.space == SpaceId::REGISTER {
                            if win_regs.contains(&inp.offset) { win_score += 1; }
                            if sysv_regs.contains(&inp.offset) { sysv_score += 1; }
                        }
                    }
                }
                if insn.ops.iter().any(|op| op.opcode == OpCode::Return) {
                    break;
                }
            }

            if win_score > 0 || sysv_score > 0 {
                let convention = if win_score > sysv_score { "__fastcall" } else { "__cdecl" };
                if let Some(func) = program.listing.get_function_mut(entry) {
                    func.calling_convention = Some(convention.into());
                }
                inferred += 1;
            }
        }

        Ok(AnalysisResult {
            analyzer_name: self.name().into(),
            functions_found: inferred,
            references_found: 0,
            instructions_decoded: 0,
        })
    }
}
