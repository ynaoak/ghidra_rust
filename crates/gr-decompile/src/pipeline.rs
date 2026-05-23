use gr_lift::{LiftedInstruction, PcodeLift};
use gr_loader::Memory;

use crate::cfg::ControlFlowGraph;
use crate::emit::CEmitter;
use crate::optimize::{run_optimization_passes, OptimizationStats};
use crate::ssa::SsaFunction;
use crate::structure::structure_cfg;

pub struct DecompileResult {
    pub c_code: String,
    pub ssa_dump: String,
    pub stats: DecompileStats,
}

pub struct DecompileStats {
    pub instructions_lifted: usize,
    pub pcode_ops: usize,
    pub basic_blocks: usize,
    pub optimization: OptimizationStats,
    pub live_ops_after: usize,
}

pub fn decompile(
    lifter: &dyn PcodeLift,
    memory: &Memory,
    entry: u64,
    func_name: &str,
    max_instructions: usize,
) -> Result<DecompileResult, String> {
    let lifted = lifter
        .lift_range(memory, entry, max_instructions)
        .map_err(|e| e.to_string())?;

    if lifted.is_empty() {
        return Err(format!("no instructions at 0x{:x}", entry));
    }

    let terminated = trim_to_return(&lifted);

    let total_pcode: usize = terminated.iter().map(|i| i.ops.len()).sum();
    let cfg = ControlFlowGraph::build(&terminated);
    let block_count = cfg.block_count();

    let mut ssa = SsaFunction::from_cfg(func_name.to_string(), entry, cfg);
    let ssa_dump = ssa.display_ssa();

    let opt_stats = run_optimization_passes(&mut ssa);
    let live_ops = ssa.live_op_count();

    let structured = structure_cfg(&ssa.cfg);
    let mut emitter = CEmitter::new();
    let c_code = emitter.emit_function(&ssa, &structured);

    Ok(DecompileResult {
        c_code,
        ssa_dump,
        stats: DecompileStats {
            instructions_lifted: terminated.len(),
            pcode_ops: total_pcode,
            basic_blocks: block_count,
            optimization: opt_stats,
            live_ops_after: live_ops,
        },
    })
}

fn trim_to_return(instructions: &[LiftedInstruction]) -> Vec<LiftedInstruction> {
    let mut result = Vec::new();
    for insn in instructions {
        let is_ret = insn
            .ops
            .iter()
            .any(|op| op.opcode == gr_core::pcode::OpCode::Return);
        result.push(insn.clone());
        if is_ret {
            break;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use gr_core::address::{Endian, SpaceId};
    use gr_lift::x86::X86Lifter;
    use gr_loader::memory::{Memory, MemoryBlock, MemoryFlags};
    use std::sync::Arc;

    fn make_memory(data: &[u8], addr: u64) -> Memory {
        let mut mem = Memory::new(SpaceId(1), Endian::Little);
        mem.add_block(MemoryBlock {
            name: ".text".into(),
            start: addr,
            size: data.len() as u64,
            flags: MemoryFlags::READ | MemoryFlags::EXECUTE,
            data: Some(Arc::from(data)),
        });
        mem
    }

    #[test]
    fn decompile_simple_function() {
        let lifter = X86Lifter::new_64();
        // push rbp; mov rbp, rsp; xor eax, eax; pop rbp; ret
        let code = [0x55, 0x48, 0x89, 0xe5, 0x31, 0xc0, 0x5d, 0xc3];
        let mem = make_memory(&code, 0x1000);

        let result = decompile(&lifter, &mem, 0x1000, "simple", 100).unwrap();
        assert!(result.c_code.contains("void simple(void)"));
        assert!(result.c_code.contains("return"));
        assert!(result.stats.instructions_lifted > 0);
        assert!(result.stats.basic_blocks >= 1);
    }

    #[test]
    fn decompile_add_function() {
        let lifter = X86Lifter::new_64();
        // sub rsp, 0x28; add rsp, 0x28; ret
        let code = [0x48, 0x83, 0xec, 0x28, 0x48, 0x83, 0xc4, 0x28, 0xc3];
        let mem = make_memory(&code, 0x1000);

        let result = decompile(&lifter, &mem, 0x1000, "stack_func", 100).unwrap();
        assert!(result.c_code.contains("void stack_func(void)"));
        assert!(result.stats.instructions_lifted == 3);
    }
}
