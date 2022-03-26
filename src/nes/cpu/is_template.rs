use super::Cpu;
use super::decoder::AddrMode;
use super::executer::FnExec;

/// 命令の外枠。複数の命令に共通するテンプレート部分。
pub struct IsTemplate {
    /// 命令の実行に必要な最大クロックサイクル数
    pub total_clock: u8,
    /// 命令のクロックごとの処理のテンプレート。
    pub fn_exec: FnExec,
    pub addr_mode: AddrMode,
}

pub const IS_TEMP_DUMMY :IsTemplate = IsTemplate {
    total_clock: u8::MAX,
    fn_exec: Cpu::fn_exec_dummy,
    addr_mode: AddrMode::Absolute,
};

pub const IS_TEMP_INDEXED_INDIRECT_X :IsTemplate = IsTemplate {
    total_clock: 6,
    fn_exec: Cpu::exec_indexed_indirect_x,
    addr_mode: AddrMode::IndexedIndirect_X,
};

pub const IS_TEMP_ZEROPAGE :IsTemplate = IsTemplate {
    total_clock: 3,
    fn_exec: Cpu::exec_zeropage,
    addr_mode: AddrMode::ZeroPage,
};

pub const IS_TEMP_IMMEDIATE :IsTemplate = IsTemplate {
    total_clock: 2,
    fn_exec: Cpu::exec_immediate,
    addr_mode: AddrMode::Immediate,
};

pub const IS_TEMP_ABSOLUTE :IsTemplate = IsTemplate {
    total_clock: 4,
    fn_exec: Cpu::exec_absolute,
    addr_mode: AddrMode::Absolute,
};

pub const IS_TEMP_INDIRECT_INDEXED_Y :IsTemplate = IsTemplate {
    total_clock: 6,
    fn_exec: Cpu::exec_indirect_indexed_y,
    addr_mode: AddrMode::IndirectIndexed_Y,
};

pub const IS_TEMP_INDEXED_ZEROPAGE_Y :IsTemplate = IsTemplate {
    total_clock: 4,
    fn_exec: Cpu::exec_indexed_zeropage_y,
    addr_mode: AddrMode::IndexedZeroPage_X,
};

pub const IS_TEMP_INDEXED_ZEROPAGE_X :IsTemplate = IsTemplate {
    total_clock: 4,
    fn_exec: Cpu::exec_indexed_zeropage_x,
    addr_mode: AddrMode::IndexedZeroPage_X,
};

pub const IS_TEMP_INDEXED_ABSOLUTE_Y :IsTemplate = IsTemplate {
    total_clock: 5,
    fn_exec: Cpu::exec_indexed_absolute_y,
    addr_mode: AddrMode::IndexedAbsolute_Y,
};

pub const IS_TEMP_INDEXED_ABSOLUTE_X :IsTemplate = IsTemplate {
    total_clock: 5,
    fn_exec: Cpu::exec_indexed_absolute_x,
    addr_mode: AddrMode::IndexedAbsolute_X,
};

pub const IS_TEMP_ZEROPAGE_RMW :IsTemplate = IsTemplate {
    total_clock: 5,
    fn_exec: Cpu::exec_zeropage_rmw,
    addr_mode: AddrMode::ZeroPage,
};

pub const IS_TEMP_ACCUMULATOR_RMW :IsTemplate = IsTemplate {
    total_clock: 2,
    fn_exec: Cpu::exec_accumulator,
    addr_mode: AddrMode::Accumulator,
};

pub const IS_TEMP_ABSOLUTE_RMW :IsTemplate = IsTemplate {
    total_clock: 6,
    fn_exec: Cpu::exec_absolute_rmw,
    addr_mode: AddrMode::Absolute,
};

pub const IS_TEMP_INDEXED_ZEROPAGE_X_RMW :IsTemplate = IsTemplate {
    total_clock: 6,
    fn_exec: Cpu::exec_indexed_zeropage_x_rmw,
    addr_mode: AddrMode::IndexedZeroPage_X,
};

pub const IS_TEMP_INDEXED_ABSOLUTE_X_RMW :IsTemplate = IsTemplate {
    total_clock: 7,
    fn_exec: Cpu::exec_indexed_absolute_x_rmw,
    addr_mode: AddrMode::IndexedAbsolute_X,
};

pub const IS_TEMP_INDIRECT_JMP :IsTemplate = IsTemplate {
    total_clock: 5,
    fn_exec: Cpu::exec_indirect_jmp,
    addr_mode: AddrMode::Indirect,
};

pub const IS_TEMP_ABSOLUTE_JMP :IsTemplate = IsTemplate {
    total_clock: 3,
    fn_exec: Cpu::exec_absolute_jmp,
    addr_mode: AddrMode::Absolute,
};

pub const IS_TEMP_RELATIVE :IsTemplate = IsTemplate {
    total_clock: 4,
    fn_exec: Cpu::exec_relative,
    addr_mode: AddrMode::Relative,
};

/// CoreとTempを一体化する
pub const IS_TEMP_JSR :IsTemplate = IsTemplate {
    total_clock: 6,
    fn_exec: Cpu::exec_jsr,
    addr_mode: AddrMode::Absolute,
};

/// CoreとTempを一体化する
pub const IS_TEMP_RTI :IsTemplate = IsTemplate {
    total_clock: 6,
    fn_exec: Cpu::exec_rti,
    addr_mode: AddrMode::Implied,
};

/// CoreとTempを一体化する
pub const IS_TEMP_RTS :IsTemplate = IsTemplate {
    total_clock: 6,
    fn_exec: Cpu::exec_rts,
    addr_mode: AddrMode::Implied,
};

pub const IS_TEMP_PUSH_STACK :IsTemplate = IsTemplate {
    total_clock: 3,
    fn_exec: Cpu::exec_push_stack,
    addr_mode: AddrMode::Implied,
};

pub const IS_TEMP_PULL_STACK :IsTemplate = IsTemplate {
    total_clock: 4,
    fn_exec: Cpu::exec_pull_stack,
    addr_mode: AddrMode::Implied,
};

pub const IS_TEMP_IMPLIED :IsTemplate = IsTemplate {
    total_clock: 2,
    fn_exec: Cpu::exec_implied,
    addr_mode: AddrMode::Implied,
};
