use std::collections::BTreeSet;

use gr_core::address::AddressSet;

#[derive(Debug, Clone)]
pub struct Function {
    pub entry_point: u64,
    pub name: String,
    pub body: AddressSet,
    pub calling_convention: Option<String>,
    pub is_thunk: bool,
    pub thunk_target: Option<u64>,
    pub call_targets: BTreeSet<u64>,
}

impl Function {
    pub fn new(entry_point: u64, name: String) -> Self {
        Self {
            entry_point,
            name,
            body: AddressSet::new(),
            calling_convention: None,
            is_thunk: false,
            thunk_target: None,
            call_targets: BTreeSet::new(),
        }
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:x} {}", self.entry_point, self.name)?;
        if self.is_thunk
            && let Some(target) = self.thunk_target {
                write!(f, " -> thunk(0x{:x})", target)?;
            }
        Ok(())
    }
}
