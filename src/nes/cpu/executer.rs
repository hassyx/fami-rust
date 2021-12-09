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

    pub fn exec_pull_stack(&mut self) {
        match self.state.counter {
            2 => (),
            3 => {
                self.inc_stack();
            }
            4 => { 
                (self.state.executer.fn_core)(self, 0);
                self.exec_finished();
            },
            _ => unreachable!(),
        };
    }

    pub fn exec_push_stack(&mut self) {
        match self.state.counter {
            2 => (),
            3 => {
                (self.state.executer.fn_core)(self, 0);
                self.exec_finished();
            },
            _ => unreachable!(),
        };
    }

    pub fn exec_rti(&mut self) {
        match self.state.counter {
            2 => (),
            3 => (),
            4 => {
                // スタックからステータスレジスタの内容を復元するが、
                // Brkフラグは実在しないので 0 にしておく。
                self.regs.p = self.pull_stack() & !Flags::BREAK.bits;
            },
            5 => self.state.op_1 = self.pull_stack(),
            6 => {
                let low = self.state.op_1;
                let high = self.pull_stack();
                self.regs.pc = make_addr(high, low);
                // 何もしないが呼んでおく。
                (self.state.executer.fn_core)(self, 0);
                self.exec_finished();
            },
            _ => unreachable!(),
        };
    }

    pub fn exec_rts(&mut self) {
        match self.state.counter {
            2 => (),
            3 => self.inc_stack(),
            4 => {
                self.state.op_1 = self.peek_stack();
                self.inc_stack();
            },
            5 => self.state.op_2 = self.peek_stack(),
            6 => {
                let low = self.state.op_1;
                let high = self.state.op_2;
                self.regs.pc = make_addr(high, low).wrapping_add(1);
                // 何もしないが呼んでおく。
                (self.state.executer.fn_core)(self, 0);
                self.exec_finished();
            },
            _ => unreachable!(),
        };
    }

    pub fn exec_jsr(&mut self) {
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => (),
            4 => {
                let high = ((self.regs.pc & 0xFF00) >> 8) as u8;
                self.push_stack(high);
            },
            5 => {
                let low = (self.regs.pc & 0x00FF) as u8;
                self.push_stack(low);
            },
            6 => {
                let low = self.state.op_1;
                let high = self.fetch();
                self.regs.pc = make_addr(high, low);
                // 何もしないが呼んでおく。
                (self.state.executer.fn_core)(self, 0);
                self.exec_finished();
            },
            _ => unreachable!(),
        };
    }
}
