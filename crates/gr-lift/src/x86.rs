use gr_core::address::{Address, SpaceId};
use iced_x86::Formatter as _;
use gr_core::pcode::{OpCode, PcodeOp, SeqNum, VarnodeData};
use gr_loader::Memory;
use smallvec::SmallVec;

use crate::lift::{LiftError, LiftedInstruction, PcodeLift};

const CONST_SPACE: SpaceId = SpaceId(0);
const RAM_SPACE: SpaceId = SpaceId(1);
const REG_SPACE: SpaceId = SpaceId(2);
const UNIQUE_SPACE: SpaceId = SpaceId(3);

fn reg(offset: u64, size: u32) -> VarnodeData {
    VarnodeData::new(REG_SPACE, offset, size)
}

fn constant(value: u64, size: u32) -> VarnodeData {
    VarnodeData::new(CONST_SPACE, value, size)
}

fn ram(offset: u64, size: u32) -> VarnodeData {
    VarnodeData::new(RAM_SPACE, offset, size)
}

fn unique(offset: u64, size: u32) -> VarnodeData {
    VarnodeData::new(UNIQUE_SPACE, offset, size)
}

fn rax(sz: u32) -> VarnodeData { reg(0x00, sz) }
fn rcx(sz: u32) -> VarnodeData { reg(0x08, sz) }
fn rdx(sz: u32) -> VarnodeData { reg(0x10, sz) }
fn rbx(sz: u32) -> VarnodeData { reg(0x18, sz) }
fn rsp(sz: u32) -> VarnodeData { reg(0x20, sz) }
fn rbp(sz: u32) -> VarnodeData { reg(0x28, sz) }
fn rsi(sz: u32) -> VarnodeData { reg(0x30, sz) }
fn rdi(sz: u32) -> VarnodeData { reg(0x38, sz) }

fn iced_reg_to_varnode(r: iced_x86::Register) -> Option<VarnodeData> {
    use iced_x86::Register;
    if r == Register::RAX { return Some(rax(8)); }
    if r == Register::EAX { return Some(rax(4)); }
    if r == Register::AX { return Some(rax(2)); }
    if r == Register::AL { return Some(rax(1)); }
    if r == Register::RCX { return Some(rcx(8)); }
    if r == Register::ECX { return Some(rcx(4)); }
    if r == Register::CX { return Some(rcx(2)); }
    if r == Register::CL { return Some(rcx(1)); }
    if r == Register::RDX { return Some(rdx(8)); }
    if r == Register::EDX { return Some(rdx(4)); }
    if r == Register::DX { return Some(rdx(2)); }
    if r == Register::DL { return Some(rdx(1)); }
    if r == Register::RBX { return Some(rbx(8)); }
    if r == Register::EBX { return Some(rbx(4)); }
    if r == Register::BX { return Some(rbx(2)); }
    if r == Register::BL { return Some(rbx(1)); }
    if r == Register::RSP { return Some(rsp(8)); }
    if r == Register::ESP { return Some(rsp(4)); }
    if r == Register::SP { return Some(rsp(2)); }
    if r == Register::RBP { return Some(rbp(8)); }
    if r == Register::EBP { return Some(rbp(4)); }
    if r == Register::BP { return Some(rbp(2)); }
    if r == Register::RSI { return Some(rsi(8)); }
    if r == Register::ESI { return Some(rsi(4)); }
    if r == Register::SI { return Some(rsi(2)); }
    if r == Register::RDI { return Some(rdi(8)); }
    if r == Register::EDI { return Some(rdi(4)); }
    if r == Register::DI { return Some(rdi(2)); }
    if r == Register::R8 { return Some(reg(0x80, 8)); }
    if r == Register::R8D { return Some(reg(0x80, 4)); }
    if r == Register::R9 { return Some(reg(0x88, 8)); }
    if r == Register::R9D { return Some(reg(0x88, 4)); }
    if r == Register::R10 { return Some(reg(0x90, 8)); }
    if r == Register::R10D { return Some(reg(0x90, 4)); }
    if r == Register::R11 { return Some(reg(0x98, 8)); }
    if r == Register::R11D { return Some(reg(0x98, 4)); }
    if r == Register::R12 { return Some(reg(0xA0, 8)); }
    if r == Register::R12D { return Some(reg(0xA0, 4)); }
    if r == Register::R13 { return Some(reg(0xA8, 8)); }
    if r == Register::R13D { return Some(reg(0xA8, 4)); }
    if r == Register::R14 { return Some(reg(0xB0, 8)); }
    if r == Register::R14D { return Some(reg(0xB0, 4)); }
    if r == Register::R15 { return Some(reg(0xB8, 8)); }
    if r == Register::R15D { return Some(reg(0xB8, 4)); }
    None
}

pub struct X86Lifter {
    is_64: bool,
}

impl X86Lifter {
    pub fn new_64() -> Self {
        Self { is_64: true }
    }

    pub fn new_32() -> Self {
        Self { is_64: false }
    }

    fn ptr_size(&self) -> u32 {
        if self.is_64 { 8 } else { 4 }
    }

    fn sp(&self) -> VarnodeData {
        rsp(self.ptr_size())
    }

    fn lift_iced(
        &self,
        insn: &iced_x86::Instruction,
        address: u64,
    ) -> Result<Vec<PcodeOp>, LiftError> {
        use iced_x86::Mnemonic::*;
        let seq = |order: u32| SeqNum::new(Address::new(RAM_SPACE, address), order);
        let mut ops: Vec<PcodeOp> = Vec::new();
        let ps = self.ptr_size();

        match insn.mnemonic() {
            Nop | Endbr64 | Endbr32 => {}

            Mov => {
                let (dst, src) = self.lift_two_operands(insn)?;
                ops.push(PcodeOp {
                    opcode: OpCode::Copy,
                    seq: seq(0),
                    output: Some(dst),
                    inputs: SmallVec::from_slice(&[src]),
                });
            }

            Push => {
                let src = self.lift_operand(insn, 0)?;
                let sp = self.sp();
                let decr = constant(src.size as u64, ps);
                // RSP = RSP - size
                ops.push(PcodeOp {
                    opcode: OpCode::IntSub,
                    seq: seq(0),
                    output: Some(sp),
                    inputs: SmallVec::from_slice(&[sp, decr]),
                });
                // STORE [RSP] = src
                ops.push(PcodeOp {
                    opcode: OpCode::Store,
                    seq: seq(1),
                    output: None,
                    inputs: SmallVec::from_slice(&[constant(RAM_SPACE.0 as u64, 4), sp, src]),
                });
            }

            Pop => {
                let dst = self.lift_operand(insn, 0)?;
                let sp = self.sp();
                // dst = LOAD [RSP]
                ops.push(PcodeOp {
                    opcode: OpCode::Load,
                    seq: seq(0),
                    output: Some(dst),
                    inputs: SmallVec::from_slice(&[constant(RAM_SPACE.0 as u64, 4), sp]),
                });
                // RSP = RSP + size
                let incr = constant(dst.size as u64, ps);
                ops.push(PcodeOp {
                    opcode: OpCode::IntAdd,
                    seq: seq(1),
                    output: Some(sp),
                    inputs: SmallVec::from_slice(&[sp, incr]),
                });
            }

            Add => {
                let (dst, src) = self.lift_two_operands(insn)?;
                ops.push(PcodeOp {
                    opcode: OpCode::IntAdd,
                    seq: seq(0),
                    output: Some(dst),
                    inputs: SmallVec::from_slice(&[dst, src]),
                });
            }

            Sub => {
                let (dst, src) = self.lift_two_operands(insn)?;
                ops.push(PcodeOp {
                    opcode: OpCode::IntSub,
                    seq: seq(0),
                    output: Some(dst),
                    inputs: SmallVec::from_slice(&[dst, src]),
                });
            }

            And => {
                let (dst, src) = self.lift_two_operands(insn)?;
                ops.push(PcodeOp {
                    opcode: OpCode::IntAnd,
                    seq: seq(0),
                    output: Some(dst),
                    inputs: SmallVec::from_slice(&[dst, src]),
                });
            }

            Or => {
                let (dst, src) = self.lift_two_operands(insn)?;
                ops.push(PcodeOp {
                    opcode: OpCode::IntOr,
                    seq: seq(0),
                    output: Some(dst),
                    inputs: SmallVec::from_slice(&[dst, src]),
                });
            }

            Xor => {
                let (dst, src) = self.lift_two_operands(insn)?;
                ops.push(PcodeOp {
                    opcode: OpCode::IntXor,
                    seq: seq(0),
                    output: Some(dst),
                    inputs: SmallVec::from_slice(&[dst, src]),
                });
            }

            Shl => {
                let (dst, src) = self.lift_two_operands(insn)?;
                ops.push(PcodeOp {
                    opcode: OpCode::IntLeft,
                    seq: seq(0),
                    output: Some(dst),
                    inputs: SmallVec::from_slice(&[dst, src]),
                });
            }

            Shr => {
                let (dst, src) = self.lift_two_operands(insn)?;
                ops.push(PcodeOp {
                    opcode: OpCode::IntRight,
                    seq: seq(0),
                    output: Some(dst),
                    inputs: SmallVec::from_slice(&[dst, src]),
                });
            }

            Sar => {
                let (dst, src) = self.lift_two_operands(insn)?;
                ops.push(PcodeOp {
                    opcode: OpCode::IntSRight,
                    seq: seq(0),
                    output: Some(dst),
                    inputs: SmallVec::from_slice(&[dst, src]),
                });
            }

            Imul => {
                if insn.op_count() >= 2 {
                    let (dst, src) = self.lift_two_operands(insn)?;
                    ops.push(PcodeOp {
                        opcode: OpCode::IntMult,
                        seq: seq(0),
                        output: Some(dst),
                        inputs: SmallVec::from_slice(&[dst, src]),
                    });
                }
            }

            Not => {
                let dst = self.lift_operand(insn, 0)?;
                ops.push(PcodeOp {
                    opcode: OpCode::IntNegate,
                    seq: seq(0),
                    output: Some(dst),
                    inputs: SmallVec::from_slice(&[dst]),
                });
            }

            Neg => {
                let dst = self.lift_operand(insn, 0)?;
                ops.push(PcodeOp {
                    opcode: OpCode::Int2Comp,
                    seq: seq(0),
                    output: Some(dst),
                    inputs: SmallVec::from_slice(&[dst]),
                });
            }

            Inc => {
                let dst = self.lift_operand(insn, 0)?;
                ops.push(PcodeOp {
                    opcode: OpCode::IntAdd,
                    seq: seq(0),
                    output: Some(dst),
                    inputs: SmallVec::from_slice(&[dst, constant(1, dst.size)]),
                });
            }

            Dec => {
                let dst = self.lift_operand(insn, 0)?;
                ops.push(PcodeOp {
                    opcode: OpCode::IntSub,
                    seq: seq(0),
                    output: Some(dst),
                    inputs: SmallVec::from_slice(&[dst, constant(1, dst.size)]),
                });
            }

            Cmp => {
                let (left, right) = self.lift_two_operands(insn)?;
                let tmp = unique(0x100, left.size);
                ops.push(PcodeOp {
                    opcode: OpCode::IntSub,
                    seq: seq(0),
                    output: Some(tmp),
                    inputs: SmallVec::from_slice(&[left, right]),
                });
                // ZF
                let zf = unique(0x200, 1);
                ops.push(PcodeOp {
                    opcode: OpCode::IntEqual,
                    seq: seq(1),
                    output: Some(zf),
                    inputs: SmallVec::from_slice(&[tmp, constant(0, tmp.size)]),
                });
                // SF
                let sf = unique(0x208, 1);
                ops.push(PcodeOp {
                    opcode: OpCode::IntSLess,
                    seq: seq(2),
                    output: Some(sf),
                    inputs: SmallVec::from_slice(&[tmp, constant(0, tmp.size)]),
                });
            }

            Test => {
                let (left, right) = self.lift_two_operands(insn)?;
                let tmp = unique(0x100, left.size);
                ops.push(PcodeOp {
                    opcode: OpCode::IntAnd,
                    seq: seq(0),
                    output: Some(tmp),
                    inputs: SmallVec::from_slice(&[left, right]),
                });
                let zf = unique(0x200, 1);
                ops.push(PcodeOp {
                    opcode: OpCode::IntEqual,
                    seq: seq(1),
                    output: Some(zf),
                    inputs: SmallVec::from_slice(&[tmp, constant(0, tmp.size)]),
                });
            }

            Lea => {
                let dst = self.lift_operand(insn, 0)?;
                if let Some(addr_val) = self.compute_lea_address(insn) {
                    ops.push(PcodeOp {
                        opcode: OpCode::Copy,
                        seq: seq(0),
                        output: Some(dst),
                        inputs: SmallVec::from_slice(&[constant(addr_val, dst.size)]),
                    });
                }
            }

            Movzx => {
                let dst = self.lift_operand(insn, 0)?;
                let src = self.lift_operand(insn, 1)?;
                ops.push(PcodeOp {
                    opcode: OpCode::IntZExt,
                    seq: seq(0),
                    output: Some(dst),
                    inputs: SmallVec::from_slice(&[src]),
                });
            }

            Movsx | Movsxd => {
                let dst = self.lift_operand(insn, 0)?;
                let src = self.lift_operand(insn, 1)?;
                ops.push(PcodeOp {
                    opcode: OpCode::IntSExt,
                    seq: seq(0),
                    output: Some(dst),
                    inputs: SmallVec::from_slice(&[src]),
                });
            }

            Call => {
                let sp = self.sp();
                let ret_addr = address + insn.len() as u64;
                // RSP = RSP - ptr_size
                ops.push(PcodeOp {
                    opcode: OpCode::IntSub,
                    seq: seq(0),
                    output: Some(sp),
                    inputs: SmallVec::from_slice(&[sp, constant(ps as u64, ps)]),
                });
                // STORE [RSP] = return_address
                ops.push(PcodeOp {
                    opcode: OpCode::Store,
                    seq: seq(1),
                    output: None,
                    inputs: SmallVec::from_slice(&[
                        constant(RAM_SPACE.0 as u64, 4),
                        sp,
                        constant(ret_addr, ps),
                    ]),
                });
                // CALL target
                let target = insn.near_branch_target();
                ops.push(PcodeOp {
                    opcode: OpCode::Call,
                    seq: seq(2),
                    output: None,
                    inputs: SmallVec::from_slice(&[ram(target, ps)]),
                });
            }

            Ret => {
                let sp = self.sp();
                let ret_tmp = unique(0x300, ps);
                // ret_tmp = LOAD [RSP]
                ops.push(PcodeOp {
                    opcode: OpCode::Load,
                    seq: seq(0),
                    output: Some(ret_tmp),
                    inputs: SmallVec::from_slice(&[constant(RAM_SPACE.0 as u64, 4), sp]),
                });
                // RSP = RSP + ptr_size
                ops.push(PcodeOp {
                    opcode: OpCode::IntAdd,
                    seq: seq(1),
                    output: Some(sp),
                    inputs: SmallVec::from_slice(&[sp, constant(ps as u64, ps)]),
                });
                // RETURN ret_tmp
                ops.push(PcodeOp {
                    opcode: OpCode::Return,
                    seq: seq(2),
                    output: None,
                    inputs: SmallVec::from_slice(&[ret_tmp]),
                });
            }

            Jmp => {
                let target = insn.near_branch_target();
                ops.push(PcodeOp {
                    opcode: OpCode::Branch,
                    seq: seq(0),
                    output: None,
                    inputs: SmallVec::from_slice(&[ram(target, ps)]),
                });
            }

            Je | Jne | Jl | Jge | Jle | Jg | Jb | Jae | Jbe | Ja
            | Js | Jns | Jo | Jno | Jp | Jnp => {
                let target = insn.near_branch_target();
                let cond = unique(0x400, 1);
                ops.push(PcodeOp {
                    opcode: OpCode::Copy,
                    seq: seq(0),
                    output: Some(cond),
                    inputs: SmallVec::from_slice(&[constant(1, 1)]),
                });
                ops.push(PcodeOp {
                    opcode: OpCode::CBranch,
                    seq: seq(1),
                    output: None,
                    inputs: SmallVec::from_slice(&[ram(target, ps), cond]),
                });
            }

            Int3 => {
                ops.push(PcodeOp {
                    opcode: OpCode::CallOther,
                    seq: seq(0),
                    output: None,
                    inputs: SmallVec::from_slice(&[constant(3, 4)]),
                });
            }

            _ => {
                ops.push(PcodeOp {
                    opcode: OpCode::CallOther,
                    seq: seq(0),
                    output: None,
                    inputs: SmallVec::from_slice(&[constant(0, 4)]),
                });
            }
        }

        Ok(ops)
    }

    fn lift_operand(&self, insn: &iced_x86::Instruction, idx: u32) -> Result<VarnodeData, LiftError> {
        use iced_x86::OpKind;
        let addr = insn.ip();
        match insn.op_kind(idx) {
            OpKind::Register => {
                iced_reg_to_varnode(insn.op_register(idx)).ok_or_else(|| LiftError::Unsupported {
                    address: addr,
                    mnemonic: format!("unsupported register {:?}", insn.op_register(idx)),
                })
            }
            OpKind::Immediate8 => Ok(constant(insn.immediate8() as u64, 1)),
            OpKind::Immediate16 => Ok(constant(insn.immediate16() as u64, 2)),
            OpKind::Immediate32 => Ok(constant(insn.immediate32() as u64, 4)),
            OpKind::Immediate64 => Ok(constant(insn.immediate64(), 8)),
            OpKind::Immediate8to16 => Ok(constant(insn.immediate8to16() as u16 as u64, 2)),
            OpKind::Immediate8to32 => Ok(constant(insn.immediate8to32() as u32 as u64, 4)),
            OpKind::Immediate8to64 => Ok(constant(insn.immediate8to64() as u64, 8)),
            OpKind::Immediate32to64 => Ok(constant(insn.immediate32to64() as u64, 8)),
            OpKind::Memory => {
                let size = insn.memory_size().size() as u32;
                if size == 0 {
                    return Ok(constant(0, self.ptr_size()));
                }
                Ok(ram(0, size))
            }
            OpKind::NearBranch16 => Ok(ram(insn.near_branch16() as u64, 2)),
            OpKind::NearBranch32 => Ok(ram(insn.near_branch32() as u64, 4)),
            OpKind::NearBranch64 => Ok(ram(insn.near_branch64(), 8)),
            _ => Err(LiftError::Unsupported {
                address: addr,
                mnemonic: format!("unsupported operand kind {:?}", insn.op_kind(idx)),
            }),
        }
    }

    fn lift_two_operands(
        &self,
        insn: &iced_x86::Instruction,
    ) -> Result<(VarnodeData, VarnodeData), LiftError> {
        let dst = self.lift_operand(insn, 0)?;
        let src = self.lift_operand(insn, 1)?;
        Ok((dst, src))
    }

    fn compute_lea_address(&self, insn: &iced_x86::Instruction) -> Option<u64> {
        if insn.op_kind(1) == iced_x86::OpKind::Memory {
            let disp = insn.memory_displacement64();
            if insn.memory_base() == iced_x86::Register::RIP {
                return Some(insn.ip().wrapping_add(insn.len() as u64).wrapping_add(disp));
            }
            if insn.memory_base() == iced_x86::Register::None
                && insn.memory_index() == iced_x86::Register::None
            {
                return Some(disp);
            }
        }
        None
    }
}

impl PcodeLift for X86Lifter {
    fn lift_instruction(
        &self,
        memory: &Memory,
        address: u64,
    ) -> Result<LiftedInstruction, LiftError> {
        let mut buf = [0u8; 15];
        let read_len = buf.len().min(
            memory
                .blocks()
                .filter(|b| b.contains(address))
                .map(|b| (b.start + b.size - address) as usize)
                .next()
                .unwrap_or(0),
        );
        if read_len == 0 {
            return Err(LiftError::UnreadableAddress(address));
        }
        memory
            .read_bytes(address, &mut buf[..read_len])
            .map_err(|_| LiftError::UnreadableAddress(address))?;

        let bitness = if self.is_64 { 64 } else { 32 };
        let mut decoder =
            iced_x86::Decoder::with_ip(bitness, &buf[..read_len], address, iced_x86::DecoderOptions::NONE);
        let mut insn = iced_x86::Instruction::default();
        decoder.decode_out(&mut insn);

        if insn.is_invalid() {
            return Err(LiftError::DecodeFailed {
                address,
                reason: "invalid instruction".into(),
            });
        }

        let mut fmt = iced_x86::IntelFormatter::new();
        let mut mnemonic = String::new();
        fmt.format(&insn, &mut mnemonic);

        let pcode_ops = self.lift_iced(&insn, address)?;

        Ok(LiftedInstruction {
            address,
            length: insn.len() as u32,
            mnemonic,
            ops: pcode_ops,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gr_core::address::Endian;
    use gr_loader::memory::{MemoryBlock, MemoryFlags};
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
    fn lift_nop() {
        let lifter = X86Lifter::new_64();
        let mem = make_memory(&[0x90], 0x1000);
        let lifted = lifter.lift_instruction(&mem, 0x1000).unwrap();
        assert_eq!(lifted.length, 1);
        assert!(lifted.ops.is_empty());
    }

    #[test]
    fn lift_push_rbp() {
        let lifter = X86Lifter::new_64();
        // push rbp = 0x55
        let mem = make_memory(&[0x55], 0x1000);
        let lifted = lifter.lift_instruction(&mem, 0x1000).unwrap();
        assert_eq!(lifted.ops.len(), 2);
        assert_eq!(lifted.ops[0].opcode, OpCode::IntSub);
        assert_eq!(lifted.ops[1].opcode, OpCode::Store);
    }

    #[test]
    fn lift_pop_rbp() {
        let lifter = X86Lifter::new_64();
        // pop rbp = 0x5d
        let mem = make_memory(&[0x5d], 0x1000);
        let lifted = lifter.lift_instruction(&mem, 0x1000).unwrap();
        assert_eq!(lifted.ops.len(), 2);
        assert_eq!(lifted.ops[0].opcode, OpCode::Load);
        assert_eq!(lifted.ops[1].opcode, OpCode::IntAdd);
    }

    #[test]
    fn lift_mov_reg_reg() {
        let lifter = X86Lifter::new_64();
        // mov rbp, rsp = 48 89 e5
        let mem = make_memory(&[0x48, 0x89, 0xe5], 0x1000);
        let lifted = lifter.lift_instruction(&mem, 0x1000).unwrap();
        assert_eq!(lifted.ops.len(), 1);
        assert_eq!(lifted.ops[0].opcode, OpCode::Copy);
    }

    #[test]
    fn lift_sub_rsp_imm() {
        let lifter = X86Lifter::new_64();
        // sub rsp, 0x28 = 48 83 ec 28
        let mem = make_memory(&[0x48, 0x83, 0xec, 0x28], 0x1000);
        let lifted = lifter.lift_instruction(&mem, 0x1000).unwrap();
        assert_eq!(lifted.ops.len(), 1);
        assert_eq!(lifted.ops[0].opcode, OpCode::IntSub);
    }

    #[test]
    fn lift_xor_self() {
        let lifter = X86Lifter::new_64();
        // xor eax, eax = 31 c0
        let mem = make_memory(&[0x31, 0xc0], 0x1000);
        let lifted = lifter.lift_instruction(&mem, 0x1000).unwrap();
        assert_eq!(lifted.ops.len(), 1);
        assert_eq!(lifted.ops[0].opcode, OpCode::IntXor);
    }

    #[test]
    fn lift_ret() {
        let lifter = X86Lifter::new_64();
        // ret = c3
        let mem = make_memory(&[0xc3], 0x1000);
        let lifted = lifter.lift_instruction(&mem, 0x1000).unwrap();
        assert_eq!(lifted.ops.len(), 3);
        assert_eq!(lifted.ops[0].opcode, OpCode::Load);
        assert_eq!(lifted.ops[1].opcode, OpCode::IntAdd);
        assert_eq!(lifted.ops[2].opcode, OpCode::Return);
    }

    #[test]
    fn lift_call() {
        let lifter = X86Lifter::new_64();
        // call rel32 = e8 10 00 00 00
        let mem = make_memory(&[0xe8, 0x10, 0x00, 0x00, 0x00], 0x1000);
        let lifted = lifter.lift_instruction(&mem, 0x1000).unwrap();
        assert_eq!(lifted.ops.len(), 3);
        assert_eq!(lifted.ops[0].opcode, OpCode::IntSub);
        assert_eq!(lifted.ops[1].opcode, OpCode::Store);
        assert_eq!(lifted.ops[2].opcode, OpCode::Call);
    }

    #[test]
    fn lift_cmp() {
        let lifter = X86Lifter::new_64();
        // cmp eax, 0 = 83 f8 00
        let mem = make_memory(&[0x83, 0xf8, 0x00], 0x1000);
        let lifted = lifter.lift_instruction(&mem, 0x1000).unwrap();
        assert_eq!(lifted.ops.len(), 3);
        assert_eq!(lifted.ops[0].opcode, OpCode::IntSub);
        assert_eq!(lifted.ops[1].opcode, OpCode::IntEqual);
        assert_eq!(lifted.ops[2].opcode, OpCode::IntSLess);
    }

    #[test]
    fn lift_je() {
        let lifter = X86Lifter::new_64();
        // je +0x10 = 74 10
        let mem = make_memory(&[0x74, 0x10], 0x1000);
        let lifted = lifter.lift_instruction(&mem, 0x1000).unwrap();
        assert_eq!(lifted.ops.len(), 2);
        assert_eq!(lifted.ops[1].opcode, OpCode::CBranch);
    }

    #[test]
    fn lift_prologue_sequence() {
        let lifter = X86Lifter::new_64();
        // push rbp; mov rbp, rsp; sub rsp, 0x28
        let code = [0x55, 0x48, 0x89, 0xe5, 0x48, 0x83, 0xec, 0x28];
        let mem = make_memory(&code, 0x1000);
        let lifted = lifter.lift_range(&mem, 0x1000, 3).unwrap();
        assert_eq!(lifted.len(), 3);
        assert!(lifted[0].mnemonic.contains("push"));
        assert!(lifted[1].mnemonic.contains("mov"));
        assert!(lifted[2].mnemonic.contains("sub"));
    }
}
