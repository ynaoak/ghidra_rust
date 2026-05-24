use gr_arch::FlowType;
use gr_program::Program;

use crate::analyzer::{AnalysisError, AnalysisResult, Analyzer};

pub struct ThunkDetectorAnalyzer;

impl Analyzer for ThunkDetectorAnalyzer {
    fn name(&self) -> &str {
        "Thunk Detector"
    }

    fn description(&self) -> &str {
        "Detects thunk functions (single unconditional jump to another function)"
    }

    fn priority(&self) -> u32 {
        450
    }

    fn analyze(&self, program: &mut Program) -> Result<AnalysisResult, AnalysisError> {
        let mut thunks_found = 0;

        let func_entries: Vec<u64> = program
            .listing
            .functions()
            .map(|f| f.entry_point)
            .collect();

        for entry in func_entries {
            let insns: Vec<_> = program
                .listing
                .instructions_in_range(entry, entry + 16)
                .collect();

            if insns.len() == 1 {
                let insn = insns[0];
                let is_jmp = insn.is_unconditional_jump()
                    || insn.flow_type == FlowType::UnconditionalJump;

                if is_jmp
                    && let Some(target) = insn.branch_target
                        && let Some(func) = program.listing.get_function_mut(entry) {
                            func.is_thunk = true;
                            func.thunk_target = Some(target);
                            thunks_found += 1;
                        }
            }
        }

        Ok(AnalysisResult {
            analyzer_name: self.name().into(),
            functions_found: thunks_found,
            references_found: 0,
            instructions_decoded: 0,
        })
    }
}

pub struct EntryPointAnalyzer;

impl Analyzer for EntryPointAnalyzer {
    fn name(&self) -> &str {
        "Entry Point"
    }

    fn description(&self) -> &str {
        "Ensures entry point is marked as a function"
    }

    fn priority(&self) -> u32 {
        10
    }

    fn analyze(&self, program: &mut Program) -> Result<AnalysisResult, AnalysisError> {
        let entry = program.entry_point();
        if entry != 0 && !program.listing.has_function(entry) {
            let name = program.function_name_at(entry);
            program
                .listing
                .add_function(gr_program::function::Function::new(entry, name));
        }
        Ok(AnalysisResult {
            analyzer_name: self.name().into(),
            functions_found: 1,
            references_found: 0,
            instructions_decoded: 0,
        })
    }
}
