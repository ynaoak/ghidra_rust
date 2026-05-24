use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub id: u32,
    pub address: u64,
    pub enabled: bool,
    pub hit_count: u64,
    pub condition: Option<BreakCondition>,
}

#[derive(Debug, Clone)]
pub enum BreakCondition {
    HitCount(u64),
    RegisterEquals { offset: u64, value: u64 },
}

impl Breakpoint {
    pub fn new(id: u32, address: u64) -> Self {
        Self {
            id,
            address,
            enabled: true,
            hit_count: 0,
            condition: None,
        }
    }

    pub fn with_condition(mut self, condition: BreakCondition) -> Self {
        self.condition = Some(condition);
        self
    }

    pub fn should_break(&self) -> bool {
        if !self.enabled {
            return false;
        }
        match &self.condition {
            None => true,
            Some(BreakCondition::HitCount(n)) => self.hit_count >= *n,
            Some(BreakCondition::RegisterEquals { .. }) => true,
        }
    }
}

#[derive(Debug, Default)]
pub struct BreakpointManager {
    breakpoints: BTreeMap<u64, Vec<Breakpoint>>,
    next_id: u32,
}

impl BreakpointManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, address: u64) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.breakpoints
            .entry(address)
            .or_default()
            .push(Breakpoint::new(id, address));
        id
    }

    pub fn add_conditional(&mut self, address: u64, condition: BreakCondition) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.breakpoints
            .entry(address)
            .or_default()
            .push(Breakpoint::new(id, address).with_condition(condition));
        id
    }

    pub fn remove(&mut self, id: u32) -> bool {
        for bps in self.breakpoints.values_mut() {
            if let Some(pos) = bps.iter().position(|b| b.id == id) {
                bps.remove(pos);
                return true;
            }
        }
        false
    }

    pub fn enable(&mut self, id: u32, enabled: bool) {
        for bps in self.breakpoints.values_mut() {
            for bp in bps.iter_mut() {
                if bp.id == id {
                    bp.enabled = enabled;
                    return;
                }
            }
        }
    }

    pub fn check(&mut self, address: u64) -> bool {
        if let Some(bps) = self.breakpoints.get_mut(&address) {
            for bp in bps.iter_mut() {
                bp.hit_count += 1;
                if bp.should_break() {
                    return true;
                }
            }
        }
        false
    }

    pub fn list(&self) -> Vec<&Breakpoint> {
        self.breakpoints.values().flat_map(|v| v.iter()).collect()
    }

    pub fn clear(&mut self) {
        self.breakpoints.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_check() {
        let mut mgr = BreakpointManager::new();
        mgr.add(0x1000);
        assert!(mgr.check(0x1000));
        assert!(!mgr.check(0x2000));
    }

    #[test]
    fn enable_disable() {
        let mut mgr = BreakpointManager::new();
        let id = mgr.add(0x1000);
        mgr.enable(id, false);
        assert!(!mgr.check(0x1000));
        mgr.enable(id, true);
        assert!(mgr.check(0x1000));
    }

    #[test]
    fn remove_breakpoint() {
        let mut mgr = BreakpointManager::new();
        let id = mgr.add(0x1000);
        assert!(mgr.remove(id));
        assert!(!mgr.check(0x1000));
    }

    #[test]
    fn hit_count_condition() {
        let mut mgr = BreakpointManager::new();
        mgr.add_conditional(0x1000, BreakCondition::HitCount(3));
        assert!(!mgr.check(0x1000));
        assert!(!mgr.check(0x1000));
        assert!(mgr.check(0x1000));
    }
}
