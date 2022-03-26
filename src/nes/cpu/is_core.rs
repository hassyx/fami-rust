use super::Cpu;
use super::executer::{FnCore, Destination};

/// 命令のコア部分。特定のレジスタへの書き込みなど。
pub struct IsCore {
    // 命令の名前(ニーモニック)
    pub name: &'static str,
    // 命令実行のコアとなる処理を実行する関数。
    pub fn_core: FnCore,
    pub dst: Destination,
}

pub const IS_DUMMY :IsCore = IsCore {
    name: "DUMMY",
    fn_core: Cpu::fn_core_dummy,
    dst: Destination::Register,
};

pub const IS_ORA :IsCore = IsCore {
    name: "ORA",
    fn_core: Cpu::ora_action,
    dst: Destination::Register,
};

pub const IS_AND :IsCore = IsCore {
    name: "AND",
    fn_core: Cpu::and_action,
    dst: Destination::Register,
};

pub const IS_EOR :IsCore = IsCore {
    name: "EOR",
    fn_core: Cpu::eor_action,
    dst: Destination::Register,
};

pub const IS_ADC :IsCore = IsCore {
    name: "ADC",
    fn_core: Cpu::adc_action,
    dst: Destination::Register,
};

pub const IS_STA :IsCore = IsCore {
    name: "STA",
    fn_core: Cpu::sta_action,
    dst: Destination::Memory,
};

pub const IS_LDA :IsCore = IsCore {
    name: "LDA",
    fn_core: Cpu::lda_action,
    dst: Destination::Register,
};

pub const IS_CMP :IsCore = IsCore {
    name: "CMP",
    fn_core: Cpu::cmp_action,
    dst: Destination::Register,
};

pub const IS_SBC :IsCore = IsCore {
    name: "SBC",
    fn_core: Cpu::sbc_action,
    dst: Destination::Register,
};

pub const IS_STX :IsCore = IsCore {
    name: "STX",
    fn_core: Cpu::stx_action,
    dst: Destination::Memory,
};

pub const IS_LDX :IsCore = IsCore {
    name: "LDX",
    fn_core: Cpu::ldx_action,
    dst: Destination::Register,
};

pub const IS_ASL :IsCore = IsCore {
    name: "ASL",
    fn_core: Cpu::asl_action,
    dst: Destination::Register,
};

pub const IS_ROL :IsCore = IsCore {
    name: "ROL",
    fn_core: Cpu::rol_action,
    dst: Destination::Register,
};

pub const IS_LSR :IsCore = IsCore {
    name: "ISR",
    fn_core: Cpu::lsr_action,
    dst: Destination::Register,
};

pub const IS_ROR :IsCore = IsCore {
    name: "ROR",
    fn_core: Cpu::ror_action,
    dst: Destination::Register,
};

pub const IS_DEC :IsCore = IsCore {
    name: "DEC",
    fn_core: Cpu::dec_action,
    dst: Destination::Register,
};

pub const IS_INC :IsCore = IsCore {
    name: "INC",
    fn_core: Cpu::inc_action,
    dst: Destination::Memory,
};

pub const IS_BIT :IsCore = IsCore {
    name: "BIT",
    fn_core: Cpu::bit_action,
    dst: Destination::Register,
};

pub const IS_JMP :IsCore = IsCore {
    name: "JMP",
    fn_core: Cpu::jmp_action,
    dst: Destination::Register,
};

pub const IS_STY :IsCore = IsCore {
    name: "STY",
    fn_core: Cpu::sty_action,
    dst: Destination::Register,
};

pub const IS_LDY :IsCore = IsCore {
    name: "LDY",
    fn_core: Cpu::ldy_action,
    dst: Destination::Register,
};

pub const IS_CPY :IsCore = IsCore {
    name: "CPY",
    fn_core: Cpu::cpy_action,
    dst: Destination::Register,
};

pub const IS_CPX :IsCore = IsCore {
    name: "CPX",
    fn_core: Cpu::cpx_action,
    dst: Destination::Register,
};

pub const IS_BPL :IsCore = IsCore {
    name: "BPL",
    fn_core: Cpu::bpl_action,
    dst: Destination::Register,
};

pub const IS_BMI :IsCore = IsCore {
    name: "BMI",
    fn_core: Cpu::bmi_action,
    dst: Destination::Register,
};

pub const IS_BVC :IsCore = IsCore {
    name: "BVC",
    fn_core: Cpu::bvc_action,
    dst: Destination::Register,
};

pub const IS_BVS :IsCore = IsCore {
    name: "BVS",
    fn_core: Cpu::bvs_action,
    dst: Destination::Register,
};

pub const IS_BCC :IsCore = IsCore {
    name: "BCC",
    fn_core: Cpu::bcc_action,
    dst: Destination::Register,
};

pub const IS_BCS :IsCore = IsCore {
    name: "BCS",
    fn_core: Cpu::bcs_action,
    dst: Destination::Register,
};

pub const IS_BNE :IsCore = IsCore {
    name: "BNE",
    fn_core: Cpu::bne_action,
    dst: Destination::Register,
};

pub const IS_BEQ :IsCore = IsCore {
    name: "BEQ",
    fn_core: Cpu::beq_action,
    dst: Destination::Register,
};

pub const IS_JSR :IsCore = IsCore {
    name: "JSR",
    fn_core: Cpu::jsr_action,
    dst: Destination::Register,
};

pub const IS_RTI :IsCore = IsCore {
    name: "RTI",
    fn_core: Cpu::rti_action,
    dst: Destination::Register,
};

pub const IS_RTS :IsCore = IsCore {
    name: "RTS",
    fn_core: Cpu::rts_action,
    dst: Destination::Register,
};

pub const IS_PHP :IsCore = IsCore {
    name: "PHP",
    fn_core: Cpu::php_action,
    dst: Destination::Register,
};

pub const IS_PLP :IsCore = IsCore {
    name: "PLP",
    fn_core: Cpu::plp_action,
    dst: Destination::Register,
};

pub const IS_PHA :IsCore = IsCore {
    name: "PHA",
    fn_core: Cpu::pha_action,
    dst: Destination::Register,
};

pub const IS_PLA :IsCore = IsCore {
    name: "PLA",
    fn_core: Cpu::pla_action,
    dst: Destination::Register,
};

pub const IS_DEY :IsCore = IsCore {
    name: "DEY",
    fn_core: Cpu::dey_action,
    dst: Destination::Register,
};

pub const IS_TAY :IsCore = IsCore {
    name: "TAY",
    fn_core: Cpu::tay_action,
    dst: Destination::Register,
};

pub const IS_INX :IsCore = IsCore {
    name: "INX",
    fn_core: Cpu::inx_action,
    dst: Destination::Register,
};

pub const IS_INY :IsCore = IsCore {
    name: "INY",
    fn_core: Cpu::iny_action,
    dst: Destination::Register,
};

pub const IS_CLC :IsCore = IsCore {
    name: "CLC",
    fn_core: Cpu::clc_action,
    dst: Destination::Register,
};

pub const IS_SEC :IsCore = IsCore {
    name: "SEC",
    fn_core: Cpu::sec_action,
    dst: Destination::Register,
};

pub const IS_CLI :IsCore = IsCore {
    name: "CLI",
    fn_core: Cpu::cli_action,
    dst: Destination::Register,
};

pub const IS_SEI :IsCore = IsCore {
    name: "SEI",
    fn_core: Cpu::sei_action,
    dst: Destination::Register,
};

pub const IS_TYA :IsCore = IsCore {
    name: "TYA",
    fn_core: Cpu::tya_action,
    dst: Destination::Register,
};

pub const IS_CLV :IsCore = IsCore {
    name: "CLV",
    fn_core: Cpu::clv_action,
    dst: Destination::Register,
};

pub const IS_CLD :IsCore = IsCore {
    name: "CLD",
    fn_core: Cpu::cld_action,
    dst: Destination::Register,
};

pub const IS_SED :IsCore = IsCore {
    name: "SED",
    fn_core: Cpu::sed_action,
    dst: Destination::Register,
};

pub const IS_TXA :IsCore = IsCore {
    name: "TXA",
    fn_core: Cpu::txa_action,
    dst: Destination::Register,
};

pub const IS_TXS :IsCore = IsCore {
    name: "TXS",
    fn_core: Cpu::txs_action,
    dst: Destination::Register,
};

pub const IS_TAX:IsCore = IsCore {
    name: "TAX",
    fn_core: Cpu::tax_action,
    dst: Destination::Register,
};

pub const IS_TSX:IsCore = IsCore {
    name: "TSX",
    fn_core: Cpu::tsx_action,
    dst: Destination::Register,
};

pub const IS_DEX:IsCore = IsCore {
    name: "DEX",
    fn_core: Cpu::dex_action,
    dst: Destination::Register,
};

pub const IS_NOP:IsCore = IsCore {
    name: "NOP",
    fn_core: Cpu::nop_action,
    dst: Destination::Register,
};
