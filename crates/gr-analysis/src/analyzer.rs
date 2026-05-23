use gr_program::Program;

pub trait Analyzer: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn priority(&self) -> u32;
    fn analyze(&self, program: &mut Program) -> Result<AnalysisResult, AnalysisError>;
}

#[derive(Debug)]
pub struct AnalysisResult {
    pub analyzer_name: String,
    pub functions_found: usize,
    pub references_found: usize,
    pub instructions_decoded: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("disassembly error: {0}")]
    Disassembly(String),
    #[error("analysis error: {0}")]
    Other(String),
}
