use std::collections::BTreeMap;

use gr_core::address::SpaceId;
use gr_core::pcode::VarnodeData;

#[derive(Debug, Clone)]
pub struct EmulatorState {
    spaces: BTreeMap<u32, SpaceData>,
}

#[derive(Debug, Clone)]
struct SpaceData {
    data: BTreeMap<u64, u8>,
}

impl SpaceData {
    fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    fn read(&self, offset: u64, size: u32) -> u64 {
        let mut result: u64 = 0;
        for i in 0..size as u64 {
            if let Some(&byte) = self.data.get(&(offset + i)) {
                result |= (byte as u64) << (i * 8);
            }
        }
        result
    }

    fn write(&mut self, offset: u64, size: u32, value: u64) {
        for i in 0..size as u64 {
            let byte = ((value >> (i * 8)) & 0xFF) as u8;
            self.data.insert(offset + i, byte);
        }
    }
}

impl EmulatorState {
    pub fn new() -> Self {
        Self {
            spaces: BTreeMap::new(),
        }
    }

    fn get_space(&self, space: SpaceId) -> Option<&SpaceData> {
        self.spaces.get(&space.0)
    }

    fn get_space_mut(&mut self, space: SpaceId) -> &mut SpaceData {
        self.spaces.entry(space.0).or_insert_with(SpaceData::new)
    }

    pub fn read_varnode(&self, vn: &VarnodeData) -> u64 {
        if vn.space == SpaceId::CONST {
            return vn.offset;
        }
        self.get_space(vn.space)
            .map(|s| s.read(vn.offset, vn.size))
            .unwrap_or(0)
    }

    pub fn write_varnode(&mut self, vn: &VarnodeData, value: u64) {
        let mask = if vn.size >= 8 {
            u64::MAX
        } else {
            (1u64 << (vn.size * 8)) - 1
        };
        self.get_space_mut(vn.space).write(vn.offset, vn.size, value & mask);
    }

    pub fn read_register(&self, offset: u64, size: u32) -> u64 {
        self.read_varnode(&VarnodeData::new(SpaceId::REGISTER, offset, size))
    }

    pub fn write_register(&mut self, offset: u64, size: u32, value: u64) {
        self.write_varnode(&VarnodeData::new(SpaceId::REGISTER, offset, size), value);
    }

    pub fn read_memory(&self, address: u64, size: u32) -> u64 {
        self.read_varnode(&VarnodeData::new(SpaceId::RAM, address, size))
    }

    pub fn write_memory(&mut self, address: u64, size: u32, value: u64) {
        self.write_varnode(&VarnodeData::new(SpaceId::RAM, address, size), value);
    }

    pub fn load_memory_bytes(&mut self, address: u64, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            self.write_memory(address + i as u64, 1, byte as u64);
        }
    }

    pub fn dump_registers(&self) -> Vec<(String, u64)> {
        let regs = [
            ("RAX", 0x00u64), ("RCX", 0x08), ("RDX", 0x10), ("RBX", 0x18),
            ("RSP", 0x20), ("RBP", 0x28), ("RSI", 0x30), ("RDI", 0x38),
            ("R8",  0x80), ("R9",  0x88), ("R10", 0x90), ("R11", 0x98),
            ("R12", 0xA0), ("R13", 0xA8), ("R14", 0xB0), ("R15", 0xB8),
        ];
        regs.iter()
            .map(|(name, off)| (name.to_string(), self.read_register(*off, 8)))
            .collect()
    }
}

impl Default for EmulatorState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write_register() {
        let mut state = EmulatorState::new();
        state.write_register(0x00, 8, 0xDEADBEEF_CAFEBABE);
        assert_eq!(state.read_register(0x00, 8), 0xDEADBEEF_CAFEBABE);
        assert_eq!(state.read_register(0x00, 4), 0xCAFEBABE);
        assert_eq!(state.read_register(0x00, 2), 0xBABE);
        assert_eq!(state.read_register(0x00, 1), 0xBE);
    }

    #[test]
    fn read_write_memory() {
        let mut state = EmulatorState::new();
        state.write_memory(0x1000, 4, 0x12345678);
        assert_eq!(state.read_memory(0x1000, 4), 0x12345678);
        assert_eq!(state.read_memory(0x1000, 2), 0x5678);
    }

    #[test]
    fn constant_varnode() {
        let state = EmulatorState::new();
        let vn = VarnodeData::new(SpaceId(0), 42, 8);
        assert_eq!(state.read_varnode(&vn), 42);
    }

    #[test]
    fn load_memory_bytes() {
        let mut state = EmulatorState::new();
        state.load_memory_bytes(0x2000, &[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(state.read_memory(0x2000, 4), 0x04030201);
    }
}
