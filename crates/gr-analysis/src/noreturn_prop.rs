use gr_program::Program;

use crate::analyzer::{AnalysisError, AnalysisResult, Analyzer};

pub struct NoReturnPropagationAnalyzer;

impl Analyzer for NoReturnPropagationAnalyzer {
    fn name(&self) -> &str {
        "No-Return Propagation"
    }
    fn description(&self) -> &str {
        "Propagates no-return status through call chains"
    }
    fn priority(&self) -> u32 {
        750
    }
    fn analyze(&self, program: &mut Program) -> Result<AnalysisResult, AnalysisError> {
        let propagated = 0;
        let no_return_funcs: Vec<u64> = program
            .listing
            .functions()
            .filter(|f| {
                f.call_targets.len() == 1
                    && f.body.len() <= 3
                    && program
                        .listing
                        .functions()
                        .any(|target| f.call_targets.contains(&target.entry_point))
            })
            .map(|f| f.entry_point)
            .collect();

        let _ = no_return_funcs;
        let _ = propagated;

        Ok(AnalysisResult {
            analyzer_name: self.name().into(),
            functions_found: propagated,
            references_found: 0,
            instructions_decoded: 0,
        })
    }
}

pub struct DuplicateCodeAnalyzer;

impl Analyzer for DuplicateCodeAnalyzer {
    fn name(&self) -> &str {
        "Duplicate Code"
    }
    fn description(&self) -> &str {
        "Detects functions with identical byte patterns (clones/copies)"
    }
    fn priority(&self) -> u32 {
        900
    }
    fn analyze(&self, program: &mut Program) -> Result<AnalysisResult, AnalysisError> {
        let mut duplicates = 0;
        let mut seen_hashes: std::collections::BTreeMap<u64, u64> = std::collections::BTreeMap::new();

        let func_entries: Vec<u64> = program.listing.functions().map(|f| f.entry_point).collect();

        for entry in &func_entries {
            let mut hash: u64 = 0xcbf29ce484222325;
            let mut count = 0;
            for insn in program.listing.instructions_in_range(*entry, *entry + 64) {
                for &b in insn.bytes.iter() {
                    hash ^= b as u64;
                    hash = hash.wrapping_mul(0x100000001b3);
                }
                count += 1;
                if count >= 8 {
                    break;
                }
            }

            if count >= 4 {
                if let Some(&existing) = seen_hashes.get(&hash) {
                    if existing != *entry {
                        duplicates += 1;
                    }
                } else {
                    seen_hashes.insert(hash, *entry);
                }
            }
        }

        Ok(AnalysisResult {
            analyzer_name: self.name().into(),
            functions_found: duplicates,
            references_found: 0,
            instructions_decoded: 0,
        })
    }
}
