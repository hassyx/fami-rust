//! Instruction decoder.

use super::Cpu;
use super::executer::*;
use super::exec_core_g1;
use super::exec_core_g2;
use super::exec_core_g3;

/// アドレッシングモード
/// TODO: matchで分岐する場合は、頻出するモードを先に置く。
#[derive(Debug, PartialEq)]
enum AddrMode {
    /// 不正なアドレッシングモード。
    // Invalid,
    /// Aレジスタに対して演算を行い、Aレジスタに格納する。
    Accumulator,
    /// オペランドの16bitの即値との演算。
    Immediate,
    /// オペランドで16bitのアドレスを指定し、参照先の8bitの値と演算を行う。
    Absolute,
    /// オペランドで8bitのアドレスを指定し、参照先の8bitの値と演算を行う。
    ZeroPage,
    /// オペランドで指定した16bitのアドレスに、レジスタXの値を足して、
    /// そのアドレスが指す8bitの値に対して演算を行う。
    IndexedAbsolute_X,
    /// オペランドで指定した16bitのアドレスに、レジスタYの値を足して、
    /// そのアドレスが指す8bitの値に対して演算を行う。
    /// 最終アドレスが16bitの最大値を超えた場合は、溢れた分を無視する。
    IndexedAbsolute_Y,
    /// オペランドで指定した8bitのアドレスに、レジスタX(一部の命令ではY)を加算して、
    /// そのアドレスが指す8bitの値に対して演算を行う。
    /// 算出したアドレスがゼロページ(0-255)を超過する、しないに関わらず、常に下位8bitの値しか見ない。
    IndexedZeroPage_X,
    /// オペランドで指定した8bitの値に、レジスタXの値を足して、ゼロページ内のアドレスを得る。
    /// 次に、このアドレスの指す8bitを下位アドレス、アドレス+1 の指す内容を上位8bitとして、
    /// 16bitの最終アドレスを得る。この最終アドレスの指す先の、8bitの値に対して操作を行う。
    /// なお、1段階目と2段階目で算出したアドレスが8bitを越える、越えないに関わらず、常に下位の8bitのみを見る。
    IndexedIndirect_X,
    /// オペランドで指定した8bitのアドレスを下位アドレス、アドレス+1 の指す内容を上位8bitとして、
    /// 16bitのアドレスを得る。このアドレスに、レジスタYの値を足して、最終アドレスを得る。
    /// 最終アドレスの指す先の8bitの値に対して操作を行う。
    /// なお、1段階目と2段階目で算出したアドレスが8bitを越える、越えないに関わらず、常に下位の8bitのみを見る。
    IndirectIndexed_Y,
}

fn panic_invalid_op(opcode: u8) -> ! {
    panic!("\"{:#0X}\" is invalid opcode.", opcode);
}

fn make_executer(fn_exec: FnExec, fn_core: FnCore, dst: Destination) -> Executer {
    Executer {
        fn_exec,
        fn_core,
        dst,
    }
}

/// OPコードをフェッチして命令種別を判定し、実行を担う関数を返す。
pub fn decode(cpu: &mut Cpu, opcode: u8) -> Executer {
    // 効率のいい命令デコードについてはここが詳しい。
    // https://llx.com/Neil/a2/opcodes.html

    // 当面は非公式命令を検出した場合にpanicさせる。
    if let Some(e) = decode_group1(opcode) {
        return e
    }
    if let Some(e) = decode_group2(opcode) {
        return e
    }
    if let Some(e) = decode_group3(opcode) {
        return e
    }

    panic_invalid_op(opcode);
}

/*  
    全命令：
    ORA AND EOR ADC STA LDA CMP SBC
    ASL ROL LSR ROR STX LDX DEC INC
*/
/// OPコードの末尾2ビットを使った解析
fn decode_group1(opcode: u8) -> Option<Executer> {
    // "aaabbbcc" で分類
    // aaa,cc = OPコード,  bbb = アドレッシングモード
    let aaa = (opcode & 0b1110_0000) >> 5;
    let bbb = (opcode & 0b0001_1100) >> 2;
    let cc = opcode & 0b0000_0011;

    if cc == 0b01 {
        let (addr_mode, fn_exec) = decode_addr_group1_01(bbb)?;
        match aaa {
            // ORA
            0b000 => Some(make_executer(fn_exec, Cpu::ora_action, Destination::Register)),
            // AND
            0b001 => Some(make_executer(fn_exec, Cpu::and_action, Destination::Register)),
            // EOR
            0b010 => Some(make_executer(fn_exec, Cpu::eor_action, Destination::Register)),
            // ADC
            0b011 => Some(make_executer(fn_exec, Cpu::adc_action, Destination::Register)),
            // STA
            0b100 => {
                // Group 1 の中では、STAのみ唯一 Immediate モードを持たない。
                if addr_mode == AddrMode::Immediate {
                    return None
                }
                Some(make_executer(fn_exec, Cpu::sta_action, Destination::Memory))
            },
            // LDA
            0b101 => Some(make_executer(fn_exec, Cpu::lda_action, Destination::Register)),
            // CMP
            0b110 => Some(make_executer(fn_exec, Cpu::cmp_action, Destination::Register)),
            // SBC
            0b111 => Some(make_executer(fn_exec, Cpu::sbc_action, Destination::Register)),
            _ => None,
        }
    } else if cc == 0b10 {
        // 注意：STXとLDXでは、IndexedZeroPage_X は Y を見る。
        // また、LDXでは、IndexedAbsolute_X は Y を見る。
        let (addr_mode, fn_exec) = decode_addr_group1_10(bbb)?;
        match aaa {
            0b000 => None,    // ASL
            0b001 => None,    // ROL
            0b010 => None,    // LSR
            0b011 => None,    // ROR
            0b100 => None,    // STX
            // LDX
            0b101 => {
                let fn_exec = match addr_mode {
                    AddrMode::IndexedZeroPage_X => Cpu::exec_indexed_zeropage_y,
                    AddrMode::IndexedAbsolute_X => Cpu::exec_indexed_absolute_y,
                    _ => fn_exec,
                };
                Some(make_executer(fn_exec, Cpu::ldx_action, Destination::Register))
            }
            0b110 => None,    // DEC
            0b111 => None,    // INC
            _ => None,
        }
    } else if cc == 0b00 {
        let (addr_mode, fn_exec) = decode_addr_group1_00(bbb)?;
        match aaa {
            0b001 => None,    //BIT
            0b010 => None,    //JMP
            0b011 => None,    //JMP (abs)
            0b100 => None,    //STY
            0b101 => None,    //LDY
            0b110 => None,    //CPY
            0b111 => None,    //CPX
            _ => None,
        }
    } else if cc == 0b11 {
        // 末尾が11の命令は存在しない
        panic_invalid_op(opcode);
    } else {
        None
    }
}

/// "aaabbbcc" 形式の命令で cc=01 の場合。
/// "bbb" を利用したアドレッシングモードのデコード。
fn decode_addr_group1_01(bbb: u8) -> Option<(AddrMode,FnExec)> {
    match bbb {
        0b000 => Some((AddrMode::IndexedIndirect_X, Cpu::exec_indexed_indirect_x)),
        0b001 => Some((AddrMode::ZeroPage, Cpu::exec_zeropage)),
        0b010 => Some((AddrMode::Immediate, Cpu::exec_immediate)),
        0b011 => Some((AddrMode::Absolute, Cpu::exec_absolute)),
        0b100 => Some((AddrMode::IndirectIndexed_Y, Cpu::exec_indirect_indexed_y)),
        0b101 => Some((AddrMode::IndexedZeroPage_X, Cpu::exec_indexed_zeropage_x)),
        0b110 => Some((AddrMode::IndexedAbsolute_Y, Cpu::exec_indexed_absolute_y)),
        0b111 => Some((AddrMode::IndexedAbsolute_X, Cpu::exec_indexed_absolute_x)),
        _ => None,
    }
}

/// "aaabbbcc" 形式の命令で cc=10 の場合。
/// "bbb" を利用したアドレッシングモードのデコード。
fn decode_addr_group1_10(bbb: u8) -> Option<(AddrMode, FnExec)> {
    match bbb {
        0b000 => Some((AddrMode::Immediate, Cpu::exec_immediate)),
        0b001 => Some((AddrMode::ZeroPage, Cpu::exec_zeropage)),
        0b010 => Some((AddrMode::Accumulator, Cpu::exec_accumulator)),
        0b011 => Some((AddrMode::Absolute, Cpu::exec_absolute)),
        0b101 => Some((AddrMode::IndexedIndirect_X, Cpu::exec_indexed_indirect_x)),
        0b111 => Some((AddrMode::IndexedAbsolute_X, Cpu::exec_indexed_absolute_x)),
        _ => None,
    }
}

/*
    全命令：
    ORA AND EOR ADC STA LDA CMP SBC
    ASL ROL LSR ROR STX LDX DEC INC
*/
/// "aaabbbcc" 形式の命令で cc=00 の場合。
/// "bbb" を利用したアドレッシングモードのデコード。
fn decode_addr_group1_00(bbb: u8) -> Option<(AddrMode, FnExec)> {
    match bbb {
        0b000 => Some((AddrMode::Immediate, Cpu::exec_immediate)),
        0b001 => Some((AddrMode::ZeroPage, Cpu::exec_zeropage)),
        0b011 => Some((AddrMode::Absolute, Cpu::exec_absolute)),
        0b101 => Some((AddrMode::IndexedIndirect_X, Cpu::exec_indexed_indirect_x)),
        0b111 => Some((AddrMode::IndexedAbsolute_X, Cpu::exec_indexed_absolute_x)),
        _ => None,
    }
}

/*
    全命令:
    BIT JMP JMP STY LDY CPY CPX
*/
/// OPコードの末尾5ビットを使った解析
fn decode_group2(opcode: u8) -> Option<Executer> {
    // "xxy10000" は全て条件付きブランチ。
    // xx = OPコード, y = 比較に用いる値
    let op = (opcode & 0b1100_0000) >> 6;
    let val = (opcode & 0b0010_0000) >> 5;
    let tail = opcode & 0b0001_1111;

    if tail != 0b0001_0000 {
        None
    } else {
        match op {
            // check negative flag
            0b00 => {
                // BPL or BMI
                None
            },
            // check overflow flag
            0b01 => {
                // BVC or BVS
                None
            },
            // check carry flag
            0b10 => {
                // BCC or BCS
                None
            },
            // check zero flag
            0b11 => {
                // BNE or BEQ
                None
            },
            _ => None,
        }
    }
}

/*
    全命令：
    BRK JSR RTI RTS PHP PLP PHA PLA DEY TAY INY INX
    CLC SEC CLI SEI TYA CLV CLD SED TXA TXS TAX TSX DEX NOP
*/
/// その他の1バイト命令をデコード
fn decode_group3(opcode: u8) -> Option<Executer> {
    match opcode {
        // BRK
        0x00 => {
            // BRKはfetchの段階で判別しているので、ここには来ない。
            unreachable!();
        }
        // JSR
        0x20 => Some(make_executer(Cpu::exec_jsr, Cpu::jsr_action, Destination::Register)),
        // RTI
        0x40 => Some(make_executer(Cpu::exec_rti, Cpu::rti_action, Destination::Register)),
        // RTS
        0x60 => Some(make_executer(Cpu::exec_rts, Cpu::rts_action, Destination::Register)),
        // PHP
        0x08 => Some(make_executer(Cpu::exec_push_stack, Cpu::php_action, Destination::Register)),
        // PLP
        0x28 => Some(make_executer(Cpu::exec_pull_stack, Cpu::plp_action, Destination::Register)),
        // PHA
        0x48 => Some(make_executer(Cpu::exec_push_stack, Cpu::pha_action, Destination::Register)),
        // PLA
        0x68 => Some(make_executer(Cpu::exec_pull_stack, Cpu::pla_action, Destination::Register)),
        // DEY
        0x88 => Some(make_executer(Cpu::exec_implied, Cpu::dey_action, Destination::Register)),
        // TAY
        0xA8 => Some(make_executer(Cpu::exec_implied, Cpu::tay_action, Destination::Register)),
        // INY
        0xC8 => Some(make_executer(Cpu::exec_implied, Cpu::inx_action, Destination::Register)),
        // INX
        0xE8 => Some(make_executer(Cpu::exec_implied, Cpu::iny_action, Destination::Register)),
        // CLC
        0x18 => Some(make_executer(Cpu::exec_implied, Cpu::clc_action, Destination::Register)),
        // SEC
        0x38 => Some(make_executer(Cpu::exec_implied, Cpu::sec_action, Destination::Register)),
        // CLI
        0x58 => Some(make_executer(Cpu::exec_implied, Cpu::cli_action, Destination::Register)),
        // SEI
        0x78 => Some(make_executer(Cpu::exec_implied, Cpu::sei_action, Destination::Register)),
        // TYA
        0x98 => Some(make_executer(Cpu::exec_implied, Cpu::tya_action, Destination::Register)),
        // CLV
        0xB8 => Some(make_executer(Cpu::exec_implied, Cpu::clv_action, Destination::Register)),
        // CLD
        0xD8 => Some(make_executer(Cpu::exec_implied, Cpu::cld_action, Destination::Register)),
        // SED
        0xF8 => Some(make_executer(Cpu::exec_implied, Cpu::sed_action, Destination::Register)),
        // TXA
        0x8A => Some(make_executer(Cpu::exec_implied, Cpu::txa_action, Destination::Register)),
        // TXS
        0x9A => Some(make_executer(Cpu::exec_implied, Cpu::txs_action, Destination::Register)),
        // TAX
        0xAA => Some(make_executer(Cpu::exec_implied, Cpu::tax_action, Destination::Register)),
        // TSX
        0xBA => Some(make_executer(Cpu::exec_implied, Cpu::tsx_action, Destination::Register)),
        // DEX
        0xCA => Some(make_executer(Cpu::exec_implied, Cpu::dex_action, Destination::Register)),
        // NOP
        0xEA => Some(make_executer(Cpu::exec_implied, Cpu::nop_action, Destination::Register)),
        _ => None,
    }
}
