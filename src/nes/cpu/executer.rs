//! Instruction executer.

use super::{Cpu, Flags};
use crate::nes::util::make_addr;
use super::is_template::*;
use super::is_core::*;
use super::instruction::Destination;

// TODO: 割り込みのポーリングのタイミングは、本来は命令の最後から2クロック前で行う。
// 現状は、命令が終了したタイミングでポーリングを解禁している。

/// 命令実行の骨組み(どの命令でも共通するテンプレート部分)の処理を担う関数
pub type FnExec = fn(cpu: &mut Cpu);
/// 命令実行処理のうち、命令ごとに異なるコア部分の処理を担う関数
pub type FnCore = fn(cpu: &mut Cpu, val: u8) -> u8;

pub struct Executer {
    /// 命令の実行に必要な合計クロックサイクル数。分岐命令の場合は動的に変化する。
    pub total_clock_var: u8,
    pub template: &'static IsTemplate,
    pub core: &'static IsCore,
}

impl Default for Executer {
    fn default() -> Self {
        Self {
            total_clock_var: 0,
            template: &IS_TEMP_DUMMY,
            core: &IS_DUMMY,
        }
    }
}

impl Cpu {

    pub fn fn_exec_dummy(&mut self) { unreachable!() }
    pub fn fn_core_dummy(&mut self, _val: u8) -> u8 { unreachable!() }
    
    pub fn exec_immediate(&mut self) {
        log::debug!("exec_immediate, counter={}", self.state.counter);
        match self.state.counter {
            2 => {
                if self.state.executer.core.dst == Destination::Register {
                    let operand = self.fetch();
                    (self.state.executer.core.fn_core)(self, operand);
                } else {
                    unreachable!("Immediate does not support read instruction.");
                }
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }

    pub fn exec_zeropage(&mut self) {
        log::debug!("exec_zeropage, counter={}", self.state.counter);
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => {
                let val;
                if self.state.executer.core.dst == Destination::Register {
                    val = self.mem.read(self.state.op_1 as u16);
                    (self.state.executer.core.fn_core)(self, val);
                } else {
                    val = (self.state.executer.core.fn_core)(self, 0);
                    self.mem.write(self.state.op_1 as u16, val);
                }
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }

    pub fn exec_indexed_zeropage_x(&mut self) {
        log::debug!("exec_indexed_zeropage_x, counter={}", self.state.counter);
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_1 = self.state.op_1.wrapping_add(self.regs.x),
            4 => {
                let addr = self.state.op_1 as u16;
                if self.state.executer.core.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.core.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.core.fn_core)(self, 0);
                    self.mem.write(addr, val);
                }
                self.exec_finished();
            }
            _ => unreachable!(),
        }
    }

    pub fn exec_indexed_zeropage_y(&mut self) {
        log::debug!("exec_indexed_zeropage_y, counter={}", self.state.counter);
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_1 = self.state.op_1.wrapping_add(self.regs.y),
            4 => {
                let addr = self.state.op_1 as u16;
                if self.state.executer.core.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.core.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.core.fn_core)(self, 0);
                    self.mem.write(addr, val);
                }
                self.exec_finished();
            }
            _ => unreachable!(),
        }
    }

    pub fn exec_absolute(&mut self) {
        log::debug!("exec_absolute, counter={}", self.state.counter);
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_2 = self.fetch(),
            4 => {
                let addr = make_addr(self.state.op_2, self.state.op_1);
                if self.state.executer.core.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.core.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.core.fn_core)(self, 0);
                    self.mem.write(addr, val);
                }
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }

    pub fn exec_implied(&mut self) {
        log::debug!("exec_implied, counter={}", self.state.counter);
        match self.state.counter {
            2 => {
                (self.state.executer.core.fn_core)(self, 0);
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }
    
    pub fn exec_indexed_absolute_x(&mut self) {
        log::debug!("exec_indexed_absolute_x, counter={}", self.state.counter);
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_2 = self.fetch(),
            4 => {
                let low = self.state.op_1;
                let high = self.state.op_2;
                let addr = make_addr(high, low).wrapping_add(self.regs.x as u16);
                if self.state.executer.core.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.core.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.core.fn_core)(self, 0);
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
        log::debug!("exec_indexed_absolute_y, counter={}", self.state.counter);
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_2 = self.fetch(),
            4 => {
                let low = self.state.op_1;
                let high = self.state.op_2;
                let addr = make_addr(high, low).wrapping_add(self.regs.y as u16);
                if self.state.executer.core.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.core.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.core.fn_core)(self, 0);
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
        log::debug!("exec_indexed_indirect_x, counter={}", self.state.counter);
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => {
                self.state.op_1 = self.state.op_1.wrapping_add(self.regs.x);
            }
            4 => {
                let low = self.mem.read(self.state.op_1 as u16);
                self.state.op_2 = low;
            },
            5 => {
                let addr = self.state.op_1.wrapping_add(1) as u16;
                let high = self.mem.read(addr);
                let low = self.state.op_2;
                self.state.addr = make_addr(high, low);
            },
            6 => {
                if self.state.executer.core.dst == Destination::Register {
                    let val = self.mem.read(self.state.addr);
                    (self.state.executer.core.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.core.fn_core)(self, 0);
                    self.mem.write(self.state.addr, val);
                }
                self.exec_finished();
            }
            _ => unreachable!(),
        }
    }

    pub fn exec_indirect_indexed_y(&mut self) {
        log::debug!("exec_indirect_indexed_y, counter={}", self.state.counter);
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
                if self.state.executer.core.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.core.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.core.fn_core)(self, 0);
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
        log::debug!("exec_pull_stack, counter={}", self.state.counter);
        match self.state.counter {
            2 => (),
            3 => {
                self.inc_stack();
            }
            4 => { 
                (self.state.executer.core.fn_core)(self, 0);
                self.exec_finished();
            },
            _ => unreachable!(),
        };
    }

    pub fn exec_push_stack(&mut self) {
        log::debug!("exec_push_stack, counter={}", self.state.counter);
        match self.state.counter {
            2 => (),
            3 => {
                (self.state.executer.core.fn_core)(self, 0);
                self.exec_finished();
            },
            _ => unreachable!(),
        };
    }

    pub fn exec_rti(&mut self) {
        log::debug!("exec_rti, counter={}", self.state.counter);
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
                (self.state.executer.core.fn_core)(self, 0);
                self.exec_finished();
            },
            _ => unreachable!(),
        };
    }

    pub fn exec_rts(&mut self) {
        log::debug!("exec_rts, counter={}", self.state.counter);
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
                (self.state.executer.core.fn_core)(self, 0);
                self.exec_finished();
            },
            _ => unreachable!(),
        };
    }

    pub fn exec_jsr(&mut self) {
        log::debug!("exec_jsr, counter={}", self.state.counter);
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
                (self.state.executer.core.fn_core)(self, 0);
                self.exec_finished();
            },
            _ => unreachable!(),
        };
    }

    pub fn exec_accumulator(&mut self) {
        log::debug!("exec_accumulator, counter={}", self.state.counter);
        match self.state.counter {
            2 => {
                let result = (self.state.executer.core.fn_core)(self, self.regs.a);
                // フラグは変更済みなので、ここでは代入するだけ
                self.regs.a = result;
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }

    /// Read-Modify-WriteなZeropageアドレッシング
    pub fn exec_zeropage_rmw(&mut self) {
        log::debug!("exec_zeropage_rmw, counter={}", self.state.counter);
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => {
                self.state.op_2 = self.mem.read(self.state.op_1 as u16);
            },
            4 => {
                self.state.op_2 = (self.state.executer.core.fn_core)(self, self.state.op_2);
            },
            5 => {
                let addr = self.state.op_1 as u16;
                let val = self.state.op_2;
                self.mem.write(addr, val);
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }

    /// Read-Modify-WriteなIndexedZeropage(X)アドレッシング
    pub fn exec_indexed_zeropage_x_rmw(&mut self) {
        log::debug!("exec_indexed_zeropage_x_rmw, counter={}", self.state.counter);
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_1 = self.state.op_1.wrapping_add(self.regs.x),
            4 => {
                self.state.addr = self.state.op_1 as u16;
                self.state.op_2 = self.mem.read(self.state.addr);
            },
            5 => {
                self.state.op_2 = (self.state.executer.core.fn_core)(self, self.state.op_2);
            },
            6 => {
                let addr = self.state.addr;
                let val = self.state.op_2;
                self.mem.write(addr, val);
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }

    /// Read-Modify-WriteなAbsoluteアドレッシング
    pub fn exec_absolute_rmw(&mut self) {
        log::debug!("exec_absolute_rmw, counter={}", self.state.counter);
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_2 = self.fetch(),
            4 => {
                let addr = make_addr(self.state.op_2, self.state.op_1);
                self.state.addr = addr;
                self.state.op_2 = self.mem.read(addr);
            },
            5 => {
                self.state.op_2 = (self.state.executer.core.fn_core)(self, self.state.op_2);
            },
            6 => {
                let addr = self.state.addr;
                let val = self.state.op_2;
                self.mem.write(addr, val);
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }

    pub fn exec_indexed_absolute_x_rmw(&mut self) {
        log::debug!("exec_indexed_absolute_x_rmw, counter={}", self.state.counter);
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => {
                let low = self.state.op_1;
                let high = self.fetch();
                self.state.addr = make_addr(high, low).wrapping_add(self.regs.x as u16);
            },
            4 => (),
            5 => self.state.op_1 = self.mem.read(self.state.addr),
            6 => self.state.op_2 = (self.state.executer.core.fn_core)(self, self.state.op_1),
            7 => {
                self.mem.write(self.state.addr, self.state.op_2);
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }

    pub fn exec_absolute_jmp(&mut self) {
        log::debug!("exec_absolute_jmp, counter={}", self.state.counter);
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => {
                let low = self.state.op_1;
                let high = self.fetch();
                self.regs.pc = make_addr(high, low);
                // 何もしないが呼んでおく
                (self.state.executer.core.fn_core)(self, 0);
                self.exec_finished();
            }
            _ => unreachable!(),
        }
    }

    pub fn exec_indirect_jmp(&mut self) {
        log::debug!("exec_indirect_jmp, counter={}", self.state.counter);
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_2 = self.fetch(),
            4 => {
                let low = self.state.op_1;
                let high = self.fetch();
                self.state.addr = make_addr(high, low);
                self.state.op_1 = self.mem.read(self.state.addr);
            },
            5 => {
                let low = self.state.op_1;
                let high = self.mem.read(self.state.addr.wrapping_add(1));
                self.regs.pc = make_addr(high, low);
                // 何もしないが呼んでおく
                (self.state.executer.core.fn_core)(self, 0);
                self.exec_finished();
            }
            _ => unreachable!(),
        }
    }

    /// 相対アドレッシングモード。このモードは分岐命令でのみ使われる。
    pub fn exec_relative(&mut self) {
        log::debug!("exec_relative, counter={}", self.state.counter);
        match self.state.counter {
            2 => {
                let offset = self.fetch();
                if (self.state.executer.core.fn_core)(self, 0) == 0 {
                    // 分岐が発生しない場合はここで終わり
                    self.exec_finished();
                } else {
                    // relativeで加算されるオペランドは符号付きなので、
                    // u8からi18へ、ビットを落とすことなく符合拡張を行う。
                    let offset = ((offset as i8) as i16) as u16;
                    // 最終的に正しい2の補数がu16として得られれば、あとは加算するだけ。
                    let addr = self.regs.pc.wrapping_add((offset as i16) as u16);
                    if (addr & 0xFF00) == (self.regs.pc & 0xFF00) {
                        // 同じページ内でジャンプするなら +1 クロック
                        self.state.op_1 = 1;
                    } else {
                        // 違うページへジャンプするなら +2 クロック
                        self.state.op_1 = 2;
                    }
                    self.state.addr = addr;
                }
            },
            3 => {
                self.state.op_1 -= 1;
                if self.state.op_1 <= 0 {
                    // 分岐が発生して、かつ同じページ内へジャンプする場合は、
                    // 例外のポーリングが発生しない。6502のバグ。
                    self.regs.pc = self.state.addr;
                    self.exec_finished();
                }
            }
            4 => {
                self.regs.pc = self.state.addr;
                self.exec_finished();
            }
            _ => unreachable!(),
        }
    }
}
