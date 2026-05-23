use std::collections::BTreeMap;

use gr_arch::DecodedInstruction;

use crate::function::Function;

#[derive(Debug, Default)]
pub struct Listing {
    instructions: BTreeMap<u64, DecodedInstruction>,
    functions: BTreeMap<u64, Function>,
}

impl Listing {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_instruction(&mut self, insn: DecodedInstruction) {
        self.instructions.insert(insn.address, insn);
    }

    pub fn get_instruction(&self, address: u64) -> Option<&DecodedInstruction> {
        self.instructions.get(&address)
    }

    pub fn has_instruction(&self, address: u64) -> bool {
        self.instructions.contains_key(&address)
    }

    pub fn instructions(&self) -> impl Iterator<Item = &DecodedInstruction> {
        self.instructions.values()
    }

    pub fn instructions_in_range(
        &self,
        start: u64,
        end: u64,
    ) -> impl Iterator<Item = &DecodedInstruction> {
        self.instructions.range(start..end).map(|(_, v)| v)
    }

    pub fn instruction_count(&self) -> usize {
        self.instructions.len()
    }

    pub fn add_function(&mut self, func: Function) {
        self.functions.insert(func.entry_point, func);
    }

    pub fn get_function(&self, entry: u64) -> Option<&Function> {
        self.functions.get(&entry)
    }

    pub fn get_function_mut(&mut self, entry: u64) -> Option<&mut Function> {
        self.functions.get_mut(&entry)
    }

    pub fn has_function(&self, entry: u64) -> bool {
        self.functions.contains_key(&entry)
    }

    pub fn functions(&self) -> impl Iterator<Item = &Function> {
        self.functions.values()
    }

    pub fn function_count(&self) -> usize {
        self.functions.len()
    }

    pub fn function_containing(&self, address: u64) -> Option<&Function> {
        self.functions
            .range(..=address)
            .next_back()
            .map(|(_, f)| f)
            .filter(|f| f.body.contains(&gr_core::address::Address::new(gr_core::address::SpaceId(1), address)))
    }
}
