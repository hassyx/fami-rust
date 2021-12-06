//! Instruction executer.

use std::ops::BitOr;

use super::{Cpu, Flags};
use crate::nes::util::make_addr;

// TODO: 割り込みのポーリングのタイミングは、本来は命令の最後から2番目で行う。
// 現状は、命令が終了したタイミングでポーリングを解禁している。

/// 命令実行の骨組み(どの命令でも共通するテンプレート部分)の処理を担う関数
pub type FnExec = fn(cpu: &mut Cpu);
/// 命令実行処理のうち、命令ごとに異なるコア部分の処理を担う関数
pub type FnCore = fn(cpu: &mut Cpu, val: u8) -> u8;

#[derive(PartialEq)]
/// 最終的な演算結果を、レジスタに書き込むのか、それともメモリに書き込むのか。
pub enum Destination {
    /// レジスタに書き込む。NOPのような書き込み対象が存在しない命令もこちらに分類する。
    Register,
    /// メモリへ書き込む。
    Memory,
}

pub struct Executer {
    pub fn_exec: FnExec,
    pub fn_core: FnCore,
    pub dst: Destination,
}

impl Default for Executer {
    fn default() -> Self {
        Self { 
            fn_exec: Cpu::fn_exec_dummy,
            fn_core: Cpu::fn_core_cummy,
            dst: Destination::Register,
        }
    }
}

impl Cpu {

    pub fn fn_exec_dummy(&mut self) { }
    pub fn fn_core_cummy(&mut self, _val: u8) -> u8 { 0 }
    
    pub fn exec_immediate(&mut self) {
        match self.state.counter {
            2 => {
                if self.state.executer.dst == Destination::Register {
                    let operand = self.fetch();
                    (self.state.executer.fn_core)(self, operand);
                } else {
                    unreachable!("Immediate does not support read instruction.");
                }
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }

    pub fn exec_zeropage(&mut self) {
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => {
                let val;
                if self.state.executer.dst == Destination::Register {
                    val = self.mem.read(self.state.op_1 as u16);
                    (self.state.executer.fn_core)(self, val);
                } else {
                    val = (self.state.executer.fn_core)(self, 0);
                    self.mem.write(self.state.op_1 as u16, val);
                }
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }

    pub fn exec_indexed_zeropage_x(&mut self) {
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_1 = self.state.op_1.wrapping_add(self.regs.x),
            4 => {
                let addr = self.state.op_1 as u16;
                if self.state.executer.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.fn_core)(self, 0);
                    self.mem.write(addr, val);
                }
                self.exec_finished();
            }
            _ => unreachable!(),
        }
    }

    pub fn exec_indexed_zeropage_y(&mut self) {
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_1 = self.state.op_1.wrapping_add(self.regs.y),
            4 => {
                let addr = self.state.op_1 as u16;
                if self.state.executer.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.fn_core)(self, 0);
                    self.mem.write(addr, val);
                }
                self.exec_finished();
            }
            _ => unreachable!(),
        }
    }

    pub fn exec_absolute(&mut self) {
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_2 = self.fetch(),
            4 => {
                let addr = make_addr(self.state.op_2, self.state.op_1);
                if self.state.executer.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.fn_core)(self, 0);
                    self.mem.write(addr, val);
                }
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }

    pub fn exec_accumulator(&mut self) {
        match self.state.counter {
            2 => {
                (self.state.executer.fn_core)(self, 0);
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }

    pub fn exec_implied(&mut self) {
        match self.state.counter {
            2 => {
                (self.state.executer.fn_core)(self, 0);
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }
    
    pub fn exec_indexed_absolute_x(&mut self) {
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_2 = self.fetch(),
            4 => {
                let low = self.state.op_1;
                let high = self.state.op_2;
                let addr = make_addr(high, low).wrapping_add(self.regs.x as u16);
                if self.state.executer.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.fn_core)(self, 0);
                    self.mem.write(addr, val);
                }
                if let Some(_) = low.checked_add(self.regs.x) {
                    self.exec_finished();
                }
            },
            5 => self.exec_finished(),
            _ => unreachable!(),
        }
    }

    pub fn exec_indexed_absolute_y(&mut self) {
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_2 = self.fetch(),
            4 => {
                let low = self.state.op_1;
                let high = self.state.op_2;
                let addr = make_addr(high, low).wrapping_add(self.regs.y as u16);
                if self.state.executer.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.fn_core)(self, 0);
                    self.mem.write(addr, val);
                }
                if let Some(_) = low.checked_add(self.regs.y) {
                    self.exec_finished();
                }
            },
            5 => self.exec_finished(),
            _ => unreachable!(),
        }
    }

    pub fn exec_indexed_indirect_x(&mut self) {
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => {
                let addr = self.state.op_1.wrapping_add(self.regs.x) as u16;
                self.state.op_1 = self.mem.read(addr);
            }
            4 => {
                let low = self.mem.read(self.state.op_1 as u16);
                self.state.op_1 = low;
            },
            5 => {
                let low = self.state.op_1;
                let addr = low.wrapping_add(1);
                let high = self.mem.read(addr as u16);
                self.state.op_2 = high;
            },
            6 => {
                let low = self.state.op_1;
                let high = self.state.op_2;
                let addr = make_addr(high, low);
                if self.state.executer.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.fn_core)(self, 0);
                    self.mem.write(addr, val);
                }
                self.exec_finished();
            }
            _ => unreachable!(),
        }
    }

    pub fn exec_indirect_indexed_y(&mut self) {
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => {
                let addr = self.state.op_1;
                let low = self.mem.read(addr as u16);
                self.state.op_2 = low;
            },
            4 => {
                let addr = self.state.op_1.wrapping_add(1);
                let high = self.mem.read(addr as u16);
                self.state.op_1 = high;
            },
            5 => {
                let high = self.state.op_1;
                let low = self.state.op_2;
                let addr = make_addr(high, low);
                let addr = addr.wrapping_add(self.regs.y as u16);
                if self.state.executer.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.fn_core)(self, 0);
                    self.mem.write(addr, val);
                }
                if let Some(_) = low.checked_add(self.regs.y) {
                    self.exec_finished();
                }
            }
            6 => self.exec_finished(),
            _ => unreachable!(),
        }
    }

    //////////////////////////////////////////////
    /// ADC (group 1):
    /// レジスタAとメモリとキャリー(もしあれば)を加算してAに格納。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + + - - +
    //////////////////////////////////////////////
    pub fn adc_action(&mut self, val: u8) -> u8 {
        log::debug!("[ADC]");
        self.regs.a_add(val);
        0
    }

    //////////////////////////////////////////////
    /// SBC (group 1):
    /// レジスタAからメモリとボロー(もしあれば)を減算してAに格納。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + + - - +
    //////////////////////////////////////////////
    pub fn sbc_action(&mut self, val: u8) -> u8 {
        log::debug!("[SBC]");
        self.regs.a_sub(val);
        0
    }

    //////////////////////////////////////////////
    /// ORA (group 1):
    /// レジスタAとメモリをORしてAに格納。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn ora_action(&mut self, val: u8) -> u8 {
        log::debug!("[ORA]");
        self.regs.a |= val;
        // コピーの結果、レジスタAのMSBが1ならNをon、0ならNをoff。
        self.regs.change_negative_by_value(self.regs.a);
        // decrementの結果、レジスタAの値が0ならZをon、それ以内ならZをoff。
        self.regs.change_zero_by_value(self.regs.a);
        0
    }

    //////////////////////////////////////////////
    /// AND (group 1):
    /// レジスタAとメモリをANDしてAに格納。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn and_action(&mut self, val: u8) -> u8 {
        log::debug!("[AND]");
        self.regs.a &= val;
        // コピーの結果、レジスタAのMSBが1ならNをon、0ならNをoff。
        self.regs.change_negative_by_value(self.regs.a);
        // decrementの結果、レジスタAの値が0ならZをon、それ以内ならZをoff。
        self.regs.change_zero_by_value(self.regs.a);
        0
    }

    //////////////////////////////////////////////
    /// EOR (group 1):
    /// レジスタAとメモリを Exclusive OR してAに格納。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn eor_action(&mut self, val: u8) -> u8 {
        log::debug!("[EOR]");
        self.regs.a ^= val;
        // コピーの結果、レジスタAのMSBが1ならNをon、0ならNをoff。
        self.regs.change_negative_by_value(self.regs.a);
        // decrementの結果、レジスタAの値が0ならZをon、それ以内ならZをoff。
        self.regs.change_zero_by_value(self.regs.a);
        0
    }

    //////////////////////////////////////////////
    /// CMP (group 1):
    /// レジスタAとメモリを比較(A - memory)し、
    /// 同じ値ならZreoをon、違うならOff。
    /// 結果のMSBが1ならNegativeをOn、0ならOff。
    /// A >= memory ならCarryをOn、そうでなければOff。
    /// なお、レジスタAの内容には影響を与えない。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + + - - -
    //////////////////////////////////////////////
    pub fn cmp_action(&mut self, val: u8) -> u8 {
        log::debug!("[CMP]");
        self.regs.a_cmp(val);
        0
    }

    //////////////////////////////////////////////
    /// LDA (group 1):
    /// 値をレジスタAにロード。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn lda_action(&mut self, val: u8) -> u8 {
        log::debug!("[LDA]");
        self.regs.a = val;
        // コピーの結果、レジスタAのMSBが1ならNをon、0ならNをoff。
        self.regs.change_negative_by_value(self.regs.a);
        // decrementの結果、レジスタAの値が0ならZをon、それ以内ならZをoff。
        self.regs.change_zero_by_value(self.regs.a);
        0
    }

    //////////////////////////////////////////////
    /// LDX (group 1):
    /// 値をレジスタXにロード。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn ldx_action(&mut self, val: u8) -> u8 {
        log::debug!("[LDX]");
        self.regs.x = val;
        // コピーの結果、レジスタXのMSBが1ならNをon、0ならNをoff。
        self.regs.change_negative_by_value(self.regs.x);
        // decrementの結果、レジスタXの値が0ならZをon、それ以内ならZをoff。
        self.regs.change_zero_by_value(self.regs.x);
        0
    }

    //////////////////////////////////////////////
    /// STA (group 1, ただしimmediateなし):
    /// レジスタAの内容をメモリに書き込む。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  - - - - - -
    //////////////////////////////////////////////
    pub fn sta_action(&mut self, _: u8) -> u8 {
        log::debug!("[STA]");
        self.regs.a
    }

    //////////////////////////////////////////////
    /// TAX (implied):
    /// レジスタAをレジスタXにコピー。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn tax_action(&mut self, _: u8) -> u8 {
        log::debug!("[TAX]");
        self.regs.x = self.regs.a;
        // コピーの結果、レジスタXのMSBが1ならNをon、0ならNをoff。
        self.regs.change_negative_by_value(self.regs.x);
        // decrementの結果、レジスタXの値が0ならZをon、それ以内ならZをoff。
        self.regs.change_zero_by_value(self.regs.x);
        0
    }

    //////////////////////////////////////////////
    /// TAY (implied):
    /// レジスタAをレジスタYにコピー。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn tay_action(&mut self, _: u8) -> u8 {
        log::debug!("[TAY]");
        self.regs.y = self.regs.a;
        // コピーの結果、レジスタYのMSBが1ならNをon、0ならNをoff。
        self.regs.change_negative_by_value(self.regs.y);
        // decrementの結果、レジスタYの値が0ならZをon、それ以内ならZをoff。
        self.regs.change_zero_by_value(self.regs.y);
        0
    }

    //////////////////////////////////////////////
    /// TXA (implied):
    /// レジスタXをレジスタAにコピー。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn txa_action(&mut self, _: u8) -> u8 {
        log::debug!("[TXA]");
        self.regs.a = self.regs.x;
        // コピーの結果、レジスタAのMSBが1ならNをon、0ならNをoff。
        self.regs.change_negative_by_value(self.regs.a);
        // decrementの結果、レジスタAの値が0ならZをon、それ以内ならZをoff。
        self.regs.change_zero_by_value(self.regs.a);
        0
    }

    //////////////////////////////////////////////
    /// TYA (implied):
    /// レジスタYをレジスタSにコピー。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn tya_action(&mut self, _: u8) -> u8 {
        log::debug!("[TYA]");
        self.regs.a = self.regs.y;
        // コピーの結果、レジスタAのMSBが1ならNをon、0ならNをoff。
        self.regs.change_negative_by_value(self.regs.a);
        // decrementの結果、レジスタAの値が0ならZをon、それ以内ならZをoff。
        self.regs.change_zero_by_value(self.regs.a);
        0
    }

    //////////////////////////////////////////////
    /// TXS (implied):
    /// レジスタXをレジスタSにコピー。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  - - - - - -
    //////////////////////////////////////////////
    pub fn txs_action(&mut self, _: u8) -> u8 {
        log::debug!("[TXS]");
        self.regs.s = self.regs.x;
        0
    }

    //////////////////////////////////////////////
    /// TSX (implied):
    /// レジスタSをレジスタXにコピー。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn tsx_action(&mut self, _: u8) -> u8 {
        log::debug!("[TSX]");
        self.regs.x = self.regs.s;
        // コピーの結果、レジスタXのMSBが1ならNをon、0ならNをoff。
        self.regs.change_negative_by_value(self.regs.x);
        // decrementの結果、レジスタXの値が0ならZをon、それ以内ならZをoff。
        self.regs.change_zero_by_value(self.regs.x);
        0
    }

    //////////////////////////////////////////////
    /// INX (Implied):
    /// レジスタXをインクリメント。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn inx_action(&mut self, _: u8) -> u8 {
        log::debug!("[INX]");
        self.regs.x = self.regs.x.wrapping_add(1);
        // incrementの結果、レジスタXのMSBが1ならNをon、0ならNをoff。
        self.regs.change_negative_by_value(self.regs.x);
        // incrementの結果、レジスタXの値が0ならZをon、それ以内ならZをoff。
        self.regs.change_zero_by_value(self.regs.x);
        0
    }

    //////////////////////////////////////////////
    /// DEX (Implied):
    /// レジスタXをデクリメント。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn dex_action(&mut self, _: u8) -> u8 {
        log::debug!("[DEX]");
        self.regs.x = self.regs.x.wrapping_sub(1);
        // decrementの結果、レジスタXのMSBが1ならNをon、0ならNをoff。
        self.regs.change_negative_by_value(self.regs.x);
        // decrementの結果、レジスタXの値が0ならZをon、それ以内ならZをoff。
        self.regs.change_zero_by_value(self.regs.x);
        0
    }

    //////////////////////////////////////////////
    /// INY (Implied):
    /// レジスタYをインクリメント。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn iny_action(&mut self, _: u8) -> u8 {
        log::debug!("[INY]");
        self.regs.x = self.regs.y.wrapping_add(1);
        // incrementの結果、レジスタYのMSBが1ならNをon、0ならNをoff。
        self.regs.change_negative_by_value(self.regs.y);
        // incrementの結果、レジスタYの値が0ならZをon、それ以内ならZをoff。
        self.regs.change_zero_by_value(self.regs.y);
        0
    }

    //////////////////////////////////////////////
    /// DEY (Implied):
    /// レジスタYをデクリメント。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn dey_action(&mut self, _: u8) -> u8 {
        log::debug!("[DEY]");
        self.regs.x = self.regs.y.wrapping_sub(1);
        // decrementの結果、レジスタYのMSBが1ならNをon、0ならNをoff。
        self.regs.change_negative_by_value(self.regs.y);
        // decrementの結果、レジスタYの値が0ならZをon、それ以内ならZをoff。
        self.regs.change_zero_by_value(self.regs.y);
        0
    }

    //////////////////////////////////////////////
    /// SEI (implied):
    /// 割り込み禁止フラグを立てる。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  - - - 1 - -
    //////////////////////////////////////////////
    pub fn sei_action(&mut self, _: u8) -> u8 {
        log::debug!("[SEI]");
        self.regs.flags_on(Flags::INT_DISABLE);
        0
    }

    //////////////////////////////////////////////
    /// CLI (implied):
    /// 割り込み禁止フラグをクリア。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  - - - 0 - -
    //////////////////////////////////////////////
    pub fn cli_action(&mut self, _: u8) -> u8 {
        log::debug!("[CLI]");
        self.regs.flags_off(Flags::INT_DISABLE);
        0
    }

    //////////////////////////////////////////////
    /// SED (implied):
    /// Decimalフラグを立てる。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  - - - - 1 -
    //////////////////////////////////////////////
    pub fn sed_action(&mut self, _: u8) -> u8 {
        log::debug!("[SED]");
        self.regs.flags_on(Flags::DECIMAL);
        0
    }

    //////////////////////////////////////////////
    /// CLD (implied):
    /// Decimalフラグをクリア。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  - - - - 0 -
    //////////////////////////////////////////////
    pub fn cld_action(&mut self, _: u8) -> u8 {
        log::debug!("[CLD]");
        self.regs.flags_off(Flags::DECIMAL);
        0
    }

    //////////////////////////////////////////////
    /// CLV (implied):
    /// Overflowフラグをクリア。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  - - - - - 0
    //////////////////////////////////////////////
    pub fn clv_action(&mut self, _: u8) -> u8 {
        log::debug!("[CLV]");
        self.regs.flags_off(Flags::OVERFLOW);
        0
    }

    //////////////////////////////////////////////
    /// SEC (implied):
    /// Carryフラグを立てる。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  - - 1 - - -
    //////////////////////////////////////////////
    pub fn sec_action(&mut self, _: u8) -> u8 {
        log::debug!("[SEC]");
        self.regs.flags_on(Flags::CARRY);
        0
    }

    //////////////////////////////////////////////
    /// CLC (implied):
    /// Carryフラグをクリア。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  - - 0 - - -
    //////////////////////////////////////////////
    pub fn clc_action(&mut self, _: u8) -> u8 {
        log::debug!("[CLC]");
        self.regs.flags_off(Flags::CARRY);
        0
    }

    //////////////////////////////////////////////
    /// NOP (impliedということにしておく):
    /// 何もしない。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  - - - - - -
    //////////////////////////////////////////////
    pub fn nop_action(&mut self, _: u8) -> u8 {
        log::debug!("[NOP]");
        0
    }

    
    
    
}
