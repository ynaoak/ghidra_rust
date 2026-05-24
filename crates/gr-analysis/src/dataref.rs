use gr_program::reference::{RefType, Reference};
use gr_program::Program;

use crate::analyzer::{AnalysisError, AnalysisResult, Analyzer};

pub struct DataReferenceAnalyzer;

impl Analyzer for DataReferenceAnalyzer {
    fn name(&self) -> &str {
        "Data Reference"
    }

    fn description(&self) -> &str {
        "Creates data references from instruction operand patterns"
    }

    fn priority(&self) -> u32 {
        500
    }

    fn analyze(&self, program: &mut Program) -> Result<AnalysisResult, AnalysisError> {
        let mut refs_found = 0;

        let valid_ranges: Vec<(u64, u64)> = program
            .info
            .sections
            .iter()
            .filter(|s| s.address != 0 && s.size > 0)
            .map(|s| (s.address, s.address + s.size))
            .collect();

        let instructions: Vec<(u64, Vec<u8>)> = program
            .listing
            .instructions()
            .map(|i| (i.address, i.bytes.to_vec()))
            .collect();

        for (addr, bytes) in &instructions {
            if bytes.len() < 4 {
                continue;
            }
            for window in bytes.windows(4) {
                let val = u32::from_le_bytes([window[0], window[1], window[2], window[3]]) as u64;
                if is_valid_data_addr(val, &valid_ranges)
                    && !program.references.get_refs_from(*addr).iter().any(|r| r.to == val)
                {
                    program.references.add(Reference::new(*addr, val, RefType::DataRead));
                    refs_found += 1;
                }
            }
        }

        Ok(AnalysisResult {
            analyzer_name: self.name().into(),
            functions_found: 0,
            references_found: refs_found,
            instructions_decoded: 0,
        })
    }
}

fn is_valid_data_addr(val: u64, ranges: &[(u64, u64)]) -> bool {
    if val < 0x1000 {
        return false;
    }
    ranges.iter().any(|&(start, end)| val >= start && val < end)
}
