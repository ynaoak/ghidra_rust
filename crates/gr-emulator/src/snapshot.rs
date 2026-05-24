use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub step: u64,
    pub address: u64,
    pub registers: BTreeMap<u64, u64>,
    pub memory_regions: Vec<MemoryRegionSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRegionSnapshot {
    pub address: u64,
    pub data: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct SnapshotManager {
    snapshots: Vec<StateSnapshot>,
    max_snapshots: usize,
}

impl SnapshotManager {
    pub fn new(max: usize) -> Self {
        Self { snapshots: Vec::new(), max_snapshots: max }
    }

    pub fn capture(&mut self, state: &crate::state::EmulatorState, step: u64, address: u64) {
        let regs = state.dump_registers()
            .into_iter()
            .filter(|(_, v)| *v != 0)
            .map(|(name, val)| {
                let offset = match name.as_str() {
                    "RAX" => 0x00, "RCX" => 0x08, "RDX" => 0x10, "RBX" => 0x18,
                    "RSP" => 0x20, "RBP" => 0x28, "RSI" => 0x30, "RDI" => 0x38,
                    "R8" => 0x80, "R9" => 0x88, "R10" => 0x90, "R11" => 0x98,
                    "R12" => 0xA0, "R13" => 0xA8, "R14" => 0xB0, "R15" => 0xB8,
                    _ => 0xFF,
                };
                (offset, val)
            })
            .collect();

        if self.snapshots.len() >= self.max_snapshots && self.max_snapshots > 0 {
            self.snapshots.remove(0);
        }
        self.snapshots.push(StateSnapshot {
            step,
            address,
            registers: regs,
            memory_regions: Vec::new(),
        });
    }

    pub fn get(&self, index: usize) -> Option<&StateSnapshot> {
        self.snapshots.get(index)
    }

    pub fn latest(&self) -> Option<&StateSnapshot> {
        self.snapshots.last()
    }

    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    pub fn restore(&self, state: &mut crate::state::EmulatorState, snapshot: &StateSnapshot) {
        for (&offset, &val) in &snapshot.registers {
            if offset != 0xFF {
                state.write_register(offset, 8, val);
            }
        }
        for region in &snapshot.memory_regions {
            state.load_memory_bytes(region.address, &region.data);
        }
    }

    pub fn save_to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(&self.snapshots)
            .map_err(|e| format!("serialize: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::EmulatorState;

    #[test]
    fn capture_and_restore() {
        let mut state = EmulatorState::new();
        state.write_register(0x00, 8, 0xDEAD);
        state.write_register(0x20, 8, 0xBEEF);

        let mut mgr = SnapshotManager::new(10);
        mgr.capture(&state, 0, 0x1000);
        assert_eq!(mgr.len(), 1);

        let mut state2 = EmulatorState::new();
        let snap = mgr.latest().unwrap();
        mgr.restore(&mut state2, snap);
        assert_eq!(state2.read_register(0x00, 8), 0xDEAD);
    }

    #[test]
    fn max_snapshots() {
        let state = EmulatorState::new();
        let mut mgr = SnapshotManager::new(3);
        for i in 0..5 {
            mgr.capture(&state, i, 0x1000 + i);
        }
        assert_eq!(mgr.len(), 3);
        assert_eq!(mgr.get(0).unwrap().step, 2);
    }
}
