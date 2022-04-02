use super::Cpu;
use super::instruction::AddrMode;
use super::executer::FnExec;

/// 命令の外枠。複数の命令に共通するテンプレート部分。
pub struct IsTemplate {
    pub name: &'static str,
    /// 命令の実行に必要な最小クロックサイクル数。
    pub min_clock: u8,
    pub fn_exec: FnExec,
    pub addr_mode: AddrMode,
}

pub const IS_TEMP_DUMMY :IsTemplate = IsTemplate {
    name: "fn_exec_dummy",
    min_clock: u8::MAX,
    fn_exec: Cpu::fn_exec_dummy,
    addr_mode: AddrMode::Absolute,
};

pub const IS_TEMP_INDEXED_INDIRECT_X :IsTemplate = IsTemplate {
    name: "exec_indexed_indirect_x",
    min_clock: 6,
    fn_exec: Cpu::exec_indexed_indirect_x,
    addr_mode: AddrMode::IndexedIndirect_X,
};

pub const IS_TEMP_ZEROPAGE :IsTemplate = IsTemplate {
    name: "exec_zeropage",
    min_clock: 3,
    fn_exec: Cpu::exec_zeropage,
    addr_mode: AddrMode::ZeroPage,
};

pub const IS_TEMP_IMMEDIATE :IsTemplate = IsTemplate {
    name: "exec_immediate",
    min_clock: 2,
    fn_exec: Cpu::exec_immediate,
    addr_mode: AddrMode::Immediate,
};

pub const IS_TEMP_ABSOLUTE :IsTemplate = IsTemplate {
    name: "exec_absolute",
    min_clock: 4,
    fn_exec: Cpu::exec_absolute,
    addr_mode: AddrMode::Absolute,
};

pub const IS_TEMP_INDIRECT_INDEXED_Y :IsTemplate = IsTemplate {
    name: "exec_indirect_indexed_y",
    min_clock: 5,
    fn_exec: Cpu::exec_indirect_indexed_y,
    addr_mode: AddrMode::IndirectIndexed_Y,
};

pub const IS_TEMP_INDEXED_ZEROPAGE_Y :IsTemplate = IsTemplate {
    name: "exec_indexed_zeropage_y",
    min_clock: 4,
    fn_exec: Cpu::exec_indexed_zeropage_y,
    addr_mode: AddrMode::IndexedZeroPage_X,
};

pub const IS_TEMP_INDEXED_ZEROPAGE_X :IsTemplate = IsTemplate {
    name: "exec_indexed_zeropage_x",
    min_clock: 4,
    fn_exec: Cpu::exec_indexed_zeropage_x,
    addr_mode: AddrMode::IndexedZeroPage_X,
};

pub const IS_TEMP_INDEXED_ABSOLUTE_Y :IsTemplate = IsTemplate {
    name: "exec_indexed_absolute_y",
    min_clock: 4,
    fn_exec: Cpu::exec_indexed_absolute_y,
    addr_mode: AddrMode::IndexedAbsolute_Y,
};

pub const IS_TEMP_INDEXED_ABSOLUTE_X :IsTemplate = IsTemplate {
    name: "exec_indexed_absolute_x",
    min_clock: 4,
    fn_exec: Cpu::exec_indexed_absolute_x,
    addr_mode: AddrMode::IndexedAbsolute_X,
};

pub const IS_TEMP_ZEROPAGE_RMW :IsTemplate = IsTemplate {
    name: "exec_zeropage_rmw",
    min_clock: 5,
    fn_exec: Cpu::exec_zeropage_rmw,
    addr_mode: AddrMode::ZeroPage,
};

pub const IS_TEMP_ACCUMULATOR_RMW :IsTemplate = IsTemplate {
    name: "exec_accumulator",
    min_clock: 2,
    fn_exec: Cpu::exec_accumulator,
    addr_mode: AddrMode::Accumulator,
};

pub const IS_TEMP_ABSOLUTE_RMW :IsTemplate = IsTemplate {
    name: "exec_absolute_rmw",
    min_clock: 6,
    fn_exec: Cpu::exec_absolute_rmw,
    addr_mode: AddrMode::Absolute,
};

pub const IS_TEMP_INDEXED_ZEROPAGE_X_RMW :IsTemplate = IsTemplate {
    name: "exec_indexed_zeropage_x_rmw",
    min_clock: 6,
    fn_exec: Cpu::exec_indexed_zeropage_x_rmw,
    addr_mode: AddrMode::IndexedZeroPage_X,
};

pub const IS_TEMP_INDEXED_ABSOLUTE_X_RMW :IsTemplate = IsTemplate {
    name: "exec_indexed_absolute_x_rmw",
    min_clock: 7,
    fn_exec: Cpu::exec_indexed_absolute_x_rmw,
    addr_mode: AddrMode::IndexedAbsolute_X,
};

pub const IS_TEMP_INDIRECT_JMP :IsTemplate = IsTemplate {
    name: "exec_indirect_jmp",
    min_clock: 5,
    fn_exec: Cpu::exec_indirect_jmp,
    addr_mode: AddrMode::Indirect,
};

pub const IS_TEMP_ABSOLUTE_JMP :IsTemplate = IsTemplate {
    name: "exec_absolute_jmp",
    min_clock: 3,
    fn_exec: Cpu::exec_absolute_jmp,
    addr_mode: AddrMode::Absolute,
};

pub const IS_TEMP_RELATIVE :IsTemplate = IsTemplate {
    name: "exec_relative",
    min_clock: 3,
    fn_exec: Cpu::exec_relative,
    addr_mode: AddrMode::Relative,
};

pub const IS_TEMP_JSR :IsTemplate = IsTemplate {
    name: "exec_jsr",
    min_clock: 6,
    fn_exec: Cpu::exec_jsr,
    addr_mode: AddrMode::Absolute,
};

pub const IS_TEMP_RTI :IsTemplate = IsTemplate {
    name: "exec_rti",
    min_clock: 6,
    fn_exec: Cpu::exec_rti,
    addr_mode: AddrMode::Implied,
};

pub const IS_TEMP_RTS :IsTemplate = IsTemplate {
    name: "exec_rts",
    min_clock: 6,
    fn_exec: Cpu::exec_rts,
    addr_mode: AddrMode::Implied,
};

pub const IS_TEMP_PUSH_STACK :IsTemplate = IsTemplate {
    name: "exec_push_stack",
    min_clock: 3,
    fn_exec: Cpu::exec_push_stack,
    addr_mode: AddrMode::Implied,
};

pub const IS_TEMP_PULL_STACK :IsTemplate = IsTemplate {
    name: "exec_pull_stack",
    min_clock: 4,
    fn_exec: Cpu::exec_pull_stack,
    addr_mode: AddrMode::Implied,
};

pub const IS_TEMP_IMPLIED :IsTemplate = IsTemplate {
    name: "exec_implied",
    min_clock: 2,
    fn_exec: Cpu::exec_implied,
    addr_mode: AddrMode::Implied,
};
