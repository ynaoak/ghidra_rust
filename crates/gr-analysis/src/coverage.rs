// Code coverage analysis: tracks which addresses have been analyzed.

use std::collections::BTreeSet;
use gr_program::Program;
use crate::analyzer::{AnalysisError, AnalysisResult, Analyzer};

#[derive(Debug, Default)]
pub struct CoverageMap {
    analyzed: BTreeSet<u64>,
    total_code_bytes: u64,
}

impl CoverageMap {
    pub fn new() -> Self { Self::default() }

    pub fn mark(&mut self, address: u64) { self.analyzed.insert(address); }
    pub fn is_analyzed(&self, address: u64) -> bool { self.analyzed.contains(&address) }
    pub fn analyzed_count(&self) -> usize { self.analyzed.len() }

    pub fn coverage_percent(&self) -> f64 {
        if self.total_code_bytes == 0 { return 0.0; }
        (self.analyzed.len() as f64 / self.total_code_bytes as f64) * 100.0
    }
}

pub struct CoverageAnalyzer;

impl Analyzer for CoverageAnalyzer {
    fn name(&self) -> &str { "Coverage" }
    fn description(&self) -> &str { "Computes analysis coverage statistics" }
    fn priority(&self) -> u32 { 999 }

    fn analyze(&self, program: &mut Program) -> Result<AnalysisResult, AnalysisError> {
        let total_code: u64 = program.info.sections.iter()
            .filter(|s| s.flags.contains(gr_loader::SectionFlags::EXECUTE))
            .map(|s| s.size)
            .sum();

        let analyzed = program.listing.instruction_count() as u64;
        let coverage = if total_code > 0 { (analyzed as f64 / total_code as f64 * 100.0) as usize } else { 0 };

        Ok(AnalysisResult {
            analyzer_name: self.name().into(),
            functions_found: coverage,
            references_found: 0,
            instructions_decoded: analyzed as usize,
        })
    }
}
