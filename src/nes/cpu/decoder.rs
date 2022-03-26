//! Instruction decoder.

use super::executer::*;
use super::is_template::*;
use super::is_core::*;
use super::instruction::AddrMode;

fn panic_invalid_op(opcode: u8) -> ! {
    panic!("\"{:#04X}\" is invalid opcode.", opcode);
}

fn make_executer(template: &'static IsTemplate, core: &'static IsCore) -> Executer {
    Executer {
        // ひとまず最大所要クロックを設定しておくが、命令内で変動する可能性がある。
        total_clock_var: template.total_clock,
        template,
        core,
    }
}

/// OPコードをフェッチして命令種別を判定し、実行を担う関数を返す。
pub fn decode(opcode: u8) -> Executer {
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
        let template = decode_addr_group1_01(bbb)?;
        match aaa {
            // ORA
            0b000 => Some(make_executer(template, &IS_ORA)),
            // AND
            0b001 => Some(make_executer(template, &IS_AND)),
            // EOR
            0b010 => Some(make_executer(template, &IS_EOR)),
            // ADC
            0b011 => Some(make_executer(template, &IS_ADC)),
            // STA
            0b100 => {
                // OPコードの末尾 "01" のグループの中では、STAのみ唯一 Immediate モードを持たない。
                if template.addr_mode == AddrMode::Immediate {
                    return None
                }
                Some(make_executer(template, &IS_STA))
            },
            // LDA
            0b101 => Some(make_executer(template, &IS_LDA)),
            // CMP
            0b110 => Some(make_executer(template, &IS_CMP)),
            // SBC
            0b111 => Some(make_executer(template, &IS_SBC)),
            _ => None,
        }
    } else if cc == 0b10 {
        if aaa == 0b100 {
            // STX
            let template = decode_addr_group1_10_stx(bbb)?;
            Some(make_executer(template, &IS_STX))
        } else if aaa == 0b101 {
            // LDX
            let template = decode_addr_group1_10_ldx(bbb)?;
            Some(make_executer(template, &IS_LDX))
        } else {
            // Read-Modify-Writeな命令
            let template = decode_addr_group1_10_rwm(bbb)?;
            match aaa {
                // ASL
                0b000 => Some(make_executer(template, &IS_ASL)),
                // ROL
                0b001 => Some(make_executer(template, &IS_ROL)),
                // LSR
                0b010 => Some(make_executer(template, &IS_LSR)),
                // ROR
                0b011 => Some(make_executer(template, &IS_ROR)),
                // DEC
                0b110 => {
                    if template.addr_mode == AddrMode::Accumulator {
                        return None
                    }
                    Some(make_executer(template, &IS_DEC))
                },
                // INC
                0b111 => {
                    if template.addr_mode == AddrMode::Accumulator {
                        return None
                    }
                    Some(make_executer(template, &IS_INC))
                }
                _ => None,
            }
        }
    } else if cc == 0b00 {
        let template = decode_addr_group1_00(bbb)?;
        match aaa {
            // BIT
            0b001 => {
                if (template.addr_mode != AddrMode::ZeroPage) && (template.addr_mode != AddrMode::Absolute) {
                    return None
                }
                Some(make_executer(template, &IS_BIT))
            }
            // JMP
            0b010 => Some(make_executer(&IS_TEMP_INDIRECT_JMP, &IS_JMP)),
            // JMP (abs)
            0b011 => Some(make_executer(&IS_TEMP_ABSOLUTE_JMP, &IS_JMP)),
            // STY
            0b100 => {
                match template.addr_mode {
                    AddrMode::ZeroPage | AddrMode::IndexedZeroPage_X | AddrMode::Absolute => {
                        Some(make_executer(template, &IS_STY))
                    },
                    _ => None
                }
            },
            // LDY
            0b101 => Some(make_executer(template, &IS_LDY)),
            // CPY
            0b110 => {
                match template.addr_mode {
                    AddrMode::Immediate | AddrMode::ZeroPage | AddrMode::Absolute => {
                        Some(make_executer(template, &IS_CPY))
                    },
                    _ => None
                }
            },
            // CPX
            0b111 => {
                match template.addr_mode {
                    AddrMode::Immediate | AddrMode::ZeroPage | AddrMode::Absolute => {
                        Some(make_executer(template, &IS_CPX))
                    },
                    _ => None
                }
            }
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
fn decode_addr_group1_01(bbb: u8) -> Option<&'static IsTemplate> {
    match bbb {
        0b000 => Some(&IS_TEMP_INDEXED_INDIRECT_X),
        0b001 => Some(&IS_TEMP_ZEROPAGE),
        0b010 => Some(&IS_TEMP_IMMEDIATE),
        0b011 => Some(&IS_TEMP_ABSOLUTE),
        0b100 => Some(&IS_TEMP_INDIRECT_INDEXED_Y),
        0b101 => Some(&IS_TEMP_INDEXED_ZEROPAGE_X),
        0b110 => Some(&IS_TEMP_INDEXED_ABSOLUTE_Y),
        0b111 => Some(&IS_TEMP_INDEXED_ABSOLUTE_X),
        _ => None,
    }
}

/// "aaabbbcc" 形式の命令で cc=10 の場合。
/// "bbb" を利用したアドレッシングモードのデコード。
fn decode_addr_group1_10_rwm(bbb: u8) -> Option<&'static IsTemplate> {
    // ここでは Read-Modified-Write なアドレッシングモードの実行関数を返す。
    // 対象となる命令は ASL,LSR,INC,DEC,ROR,ROL.
    match bbb {
        0b000 => None, // Immediateな命令は存在しない
        0b001 => Some(&IS_TEMP_ZEROPAGE_RMW),
        0b010 => Some(&IS_TEMP_ACCUMULATOR_RMW),
        0b011 => Some(&IS_TEMP_ABSOLUTE_RMW),
        0b101 => Some(&IS_TEMP_INDEXED_ZEROPAGE_X_RMW),
        0b111 => Some(&IS_TEMP_INDEXED_ABSOLUTE_X_RMW),
        _ => None,
    }
}

/// STX用。"aaabbbcc" 形式の命令で cc=10 の場合。
fn decode_addr_group1_10_stx(bbb: u8) -> Option<&'static IsTemplate> {
    match bbb {
        0b000 => None,  // Immediateは無し
        0b001 => Some(&IS_TEMP_ZEROPAGE),
        0b010 => None,  // Accumulatorは無し
        0b011 => Some(&IS_TEMP_ABSOLUTE),
        // STXでは、IndexedZeroPage_X で Y を見る。
        0b101 => Some(&IS_TEMP_INDEXED_ZEROPAGE_Y),
        0b111 => None,  // IndexedAbsolute_Xは無し
        _ => None,
    }
}

/// LDX用。"aaabbbcc" 形式の命令で cc=10 の場合。
fn decode_addr_group1_10_ldx(bbb: u8) -> Option<&'static IsTemplate> {
    match bbb {
        0b000 => Some(&IS_TEMP_IMMEDIATE),
        0b001 => Some(&IS_TEMP_ZEROPAGE),
        0b010 => None,  // Accumulatorは無し
        0b011 => Some(&IS_TEMP_ABSOLUTE),
        // LDXでは、IndexedZeroPage_X は Y を見る。
        0b101 => Some(&IS_TEMP_INDEXED_ZEROPAGE_Y),
        // LDXでは、IndexedAbsolute_X は Y を見る。
        0b111 => Some(&IS_TEMP_INDEXED_ABSOLUTE_Y),
        _ => None,
    }
}

/*
    全命令：
    BIT JMP JMP(abs) STY LDY CPY CPX
*/
/// "aaabbbcc" 形式の命令で cc=00 の場合。
/// "bbb" を利用したアドレッシングモードのデコード。
fn decode_addr_group1_00(bbb: u8) -> Option<&'static IsTemplate> {
    match bbb {
        0b000 => Some(&IS_TEMP_ABSOLUTE),
        0b001 => Some(&IS_TEMP_ZEROPAGE),
        0b011 => Some(&IS_TEMP_ABSOLUTE),
        0b101 => Some(&IS_TEMP_INDEXED_INDIRECT_X),
        0b111 => Some(&IS_TEMP_INDEXED_ABSOLUTE_X),
        _ => None,
    }
}

/*
    全命令:
    BPL BMI BVC BVS BCC BCS BNE BEQ
*/
/// OPコードの末尾5ビットを使った解析
fn decode_group2(opcode: u8) -> Option<Executer> {
    // "xxy10000" は全て条件付きブランチ命令。
    // xx = OPコード, y = 比較に用いる値
    let op   = (opcode & 0b1100_0000) >> 6;
    let val  = (opcode & 0b0010_0000) >> 5;
    let tail = opcode & 0b0001_1111;

    if tail != 0b0001_0000 {
        None
    } else {
        match op {
            // check negative flag
            0b00 => {
                if val == 0 {
                    // BPL
                    Some(make_executer(&IS_TEMP_RELATIVE, &IS_BPL))
                } else {
                    //BMI
                    Some(make_executer(&IS_TEMP_RELATIVE, &IS_BMI))
                }
            },
            // check overflow flag
            0b01 => {
                if val == 0 {
                    // BVC
                    Some(make_executer(&IS_TEMP_RELATIVE, &IS_BVC))
                } else {
                    // BVS
                    Some(make_executer(&IS_TEMP_RELATIVE, &IS_BVS))
                }
            },
            // check carry flag
            0b10 => {
                if val == 0 {
                    // BCC
                    Some(make_executer(&IS_TEMP_RELATIVE, &IS_BCC))
                } else {
                    // BCS
                    Some(make_executer(&IS_TEMP_RELATIVE, &IS_BCS))
                }
            },
            // check zero flag
            0b11 => {
                if val == 0 {
                    // BNE
                    Some(make_executer(&IS_TEMP_RELATIVE, &IS_BNE))
                } else {
                    // BEQ
                    Some(make_executer(&IS_TEMP_RELATIVE, &IS_BEQ))
                }
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
        // JSR
        0x20 => Some(make_executer(&IS_TEMP_JSR, &IS_JSR)),
        // RTI
        0x40 => Some(make_executer(&IS_TEMP_RTI, &IS_RTI)),
        // RTS
        0x60 => Some(make_executer(&IS_TEMP_RTS, &IS_RTS)),
        // PHP
        0x08 => Some(make_executer(&IS_TEMP_PUSH_STACK, &IS_PHP)),
        // PLP
        0x28 => Some(make_executer(&IS_TEMP_PULL_STACK, &IS_PLP)),
        // PHA
        0x48 => Some(make_executer(&IS_TEMP_PUSH_STACK, &IS_PHA)),
        // PLA
        0x68 => Some(make_executer(&IS_TEMP_PULL_STACK, &IS_PLA)),
        // DEY
        0x88 => Some(make_executer(&IS_TEMP_IMPLIED, &IS_DEY)),
        // TAY
        0xA8 => Some(make_executer(&IS_TEMP_IMPLIED, &IS_TAY)),
        // INY
        0xC8 => Some(make_executer(&IS_TEMP_IMPLIED, &IS_INX)),
        // INX
        0xE8 => Some(make_executer(&IS_TEMP_IMPLIED, &IS_INY)),
        // CLC
        0x18 => Some(make_executer(&IS_TEMP_IMPLIED, &IS_CLC)),
        // SEC
        0x38 => Some(make_executer(&IS_TEMP_IMPLIED, &IS_SEC)),
        // CLI
        0x58 => Some(make_executer(&IS_TEMP_IMPLIED, &IS_CLI)),
        // SEI
        0x78 => Some(make_executer(&IS_TEMP_IMPLIED, &IS_SEI)),
        // TYA
        0x98 => Some(make_executer(&IS_TEMP_IMPLIED, &IS_TYA)),
        // CLV
        0xB8 => Some(make_executer(&IS_TEMP_IMPLIED, &IS_CLV)),
        // CLD
        0xD8 => Some(make_executer(&IS_TEMP_IMPLIED, &IS_CLD)),
        // SED
        0xF8 => Some(make_executer(&IS_TEMP_IMPLIED, &IS_SED)),
        // TXA
        0x8A => Some(make_executer(&IS_TEMP_IMPLIED, &IS_TXA)),
        // TXS
        0x9A => Some(make_executer(&IS_TEMP_IMPLIED, &IS_TXS)),
        // TAX
        0xAA => Some(make_executer(&IS_TEMP_IMPLIED, &IS_TAX)),
        // TSX
        0xBA => Some(make_executer(&IS_TEMP_IMPLIED, &IS_TSX)),
        // DEX
        0xCA => Some(make_executer(&IS_TEMP_IMPLIED, &IS_DEX)),
        // NOP
        0xEA => Some(make_executer(&IS_TEMP_IMPLIED, &IS_NOP)),
        // BRK
        // BRKはfetchの段階で判別しているので、ここには来ない。
        // 0x00 => unreachable!(),
        _ => None,
    }
}
