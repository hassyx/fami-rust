//! Instruction executer.

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
    Register,
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
    /// ORA: レジスタAとメモリをORしてAに格納。
    //////////////////////////////////////////////
    pub fn ora_action(&mut self, val: u8) -> u8 {
        log::debug!("[ORA]");
        self.regs.a |= val;
        0
    }

    //////////////////////////////////////////////
    /// LDA: 値をレジスタAにロード。
    //////////////////////////////////////////////
    pub fn lda_action(&mut self, val: u8) -> u8 {
        log::debug!("[LDA]");
        self.regs.a = val;
        0
    }

    //////////////////////////////////////////////
    /// LDA: 値をレジスタXにロード。
    //////////////////////////////////////////////
    pub fn ldx_action(&mut self, val: u8) -> u8 {
        log::debug!("[LDX]");
        self.regs.x = val;
        0
    }

    //////////////////////////////////////////////
    /// STA: レジスタAの内容をメモリに書き込む。
    //////////////////////////////////////////////
    pub fn sta_action(&mut self, _: u8) -> u8 {
        log::debug!("[STA]");
        self.regs.a
    }

    //////////////////////////////////////////////
    /// TAX: レジスタAをレジスタXにコピー。
    //////////////////////////////////////////////
    pub fn tax_action(&mut self, _: u8) -> u8 {
        log::debug!("[TAX]");
        self.regs.x = self.regs.a;
        0
    }

    //////////////////////////////////////////////
    /// TAY: レジスタAをレジスタYにコピー。
    //////////////////////////////////////////////
    pub fn tay_action(&mut self, _: u8) -> u8 {
        log::debug!("[TAY]");
        self.regs.y = self.regs.a;
        0
    }

    //////////////////////////////////////////////
    /// TXA: レジスタXをレジスタAにコピー。
    //////////////////////////////////////////////
    pub fn txa_action(&mut self, _: u8) -> u8 {
        log::debug!("[TXA]");
        self.regs.a = self.regs.x;
        0
    }

    //////////////////////////////////////////////
    /// TYA: レジスタYをレジスタSにコピー。
    //////////////////////////////////////////////
    pub fn tya_action(&mut self, _: u8) -> u8 {
        log::debug!("[TYA]");
        self.regs.a = self.regs.y;
        0
    }

    //////////////////////////////////////////////
    /// TXS: レジスタXをレジスタSにコピー。
    //////////////////////////////////////////////
    pub fn txs_action(&mut self, _: u8) -> u8 {
        log::debug!("[TXS]");
        self.regs.s = self.regs.x;
        0
    }

    //////////////////////////////////////////////
    /// TXS: レジスタSをレジスタXにコピー。
    //////////////////////////////////////////////
    pub fn tsx_action(&mut self, _: u8) -> u8 {
        log::debug!("[TSX]");
        self.regs.x = self.regs.s;
        0
    }

    //////////////////////////////////////////////
    /// DEX: レジスタXをデクリメント。
    //////////////////////////////////////////////
    pub fn dex_action(&mut self, _: u8) -> u8 {
        log::debug!("[DEX]");
        //・オーバーフローするのか？？
        //・あと今までに実装した命令でフラグが変化する場合は修正が必要。
        self.regs.x = self.regs.x.wrapping_sub(1);
        0
    }



    

    //////////////////////////////////////////////
    /// SEI: 割り込み禁止フラグを立てる。
    //////////////////////////////////////////////
    pub fn sei_action(&mut self, _: u8) -> u8 {
        log::debug!("[SEI]");
        self.regs.flags_on(Flags::INT_DISABLE);
        0
    }

    //////////////////////////////////////////////
    /// SEI: 割り込み禁止フラグをクリア。
    //////////////////////////////////////////////
    pub fn cli_action(&mut self, _: u8) -> u8 {
        log::debug!("[CLI]");
        self.regs.flags_off(Flags::INT_DISABLE);
        0
    }

    //////////////////////////////////////////////
    /// SED: Decimalフラグを立てる。
    //////////////////////////////////////////////
    pub fn sed_action(&mut self, _: u8) -> u8 {
        log::debug!("[SED]");
        self.regs.flags_on(Flags::DECIMAL);
        0
    }

    //////////////////////////////////////////////
    /// CLD: Decimalフラグをクリア。
    //////////////////////////////////////////////
    pub fn cld_action(&mut self, _: u8) -> u8 {
        log::debug!("[CLD]");
        self.regs.flags_off(Flags::DECIMAL);
        0
    }

    //////////////////////////////////////////////
    /// CLV: Overflowフラグをクリア。
    //////////////////////////////////////////////
    pub fn clv_action(&mut self, _: u8) -> u8 {
        log::debug!("[CLV]");
        self.regs.flags_off(Flags::OVERFLOW);
        0
    }

    //////////////////////////////////////////////
    /// SEC: Carryフラグを立てる。
    //////////////////////////////////////////////
    pub fn sec_action(&mut self, _: u8) -> u8 {
        log::debug!("[SEC]");
        self.regs.flags_on(Flags::CARRY);
        0
    }

    //////////////////////////////////////////////
    /// CLV: Carryフラグをクリア。
    //////////////////////////////////////////////
    pub fn clc_action(&mut self, _: u8) -> u8 {
        log::debug!("[CLC]");
        self.regs.flags_off(Flags::CARRY);
        0
    }

    
    
    
}
