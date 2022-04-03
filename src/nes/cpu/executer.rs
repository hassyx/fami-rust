//! Instruction executer.

use super::{Cpu, Flags, IntType};
use crate::nes::util::make_addr;
use super::instruction::*;

// TODO: 割り込みのポーリングのタイミングは、本来は命令の最後から2クロック前で行う。
// 現状は、命令が終了したタイミングでポーリングを解禁している。

/// 命令実行の骨組み(どの命令でも共通するテンプレート部分)の処理を担う関数
pub type FnExec = fn(cpu: &mut Cpu);
/// 命令実行処理のうち、命令ごとに異なるコア部分の処理を担う関数
pub type FnCore = fn(cpu: &mut Cpu, val: u8) -> u8;

pub struct Executer {
    /// 命令が完了する最小クロックサイクル数。
    /// 分岐命令や、ページをまたぐメモリアクセスが発生した場合に、
    /// 動的に増加する場合がある。
    pub last_cycle: u8,
    pub inst: &'static Instruction,
}

impl Default for Executer {
    fn default() -> Self {
        Self {
            last_cycle: 0,
            inst: &DUMMY_INSTRUCTION,
        }
    }
}

impl Cpu {

    pub fn fn_exec_dummy(&mut self) { unreachable!() }
    pub fn fn_core_dummy(&mut self, _val: u8) -> u8 { unreachable!() }
    
    pub fn exec_immediate(&mut self) {
        match self.state.counter {
            2 => {
                if self.state.executer.inst.dst == Destination::Register {
                    let operand = self.fetch();
                    (self.state.executer.inst.fn_core)(self, operand);
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
                if self.state.executer.inst.dst == Destination::Register {
                    let val = self.mem.read(self.state.op_1 as u16);
                    (self.state.executer.inst.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.inst.fn_core)(self, 0);
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
                if self.state.executer.inst.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.inst.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.inst.fn_core)(self, 0);
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
                if self.state.executer.inst.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.inst.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.inst.fn_core)(self, 0);
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
                if self.state.executer.inst.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.inst.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.inst.fn_core)(self, 0);
                    self.mem.write(addr, val);
                }
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }

    pub fn exec_implied(&mut self) {
        match self.state.counter {
            2 => {
                (self.state.executer.inst.fn_core)(self, 0);
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
                if self.state.executer.inst.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.inst.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.inst.fn_core)(self, 0);
                    self.mem.write(addr, val);
                }
                if let Some(_) = low.checked_add(self.regs.x) {
                    self.exec_finished();
                } else {
                    self.state.executer.last_cycle += 1;
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
                if self.state.executer.inst.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.inst.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.inst.fn_core)(self, 0);
                    self.mem.write(addr, val);
                }
                if let Some(_) = low.checked_add(self.regs.y) {
                    self.exec_finished();
                } else {
                    self.state.executer.last_cycle += 1;
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
                if self.state.executer.inst.dst == Destination::Register {
                    let val = self.mem.read(self.state.addr);
                    (self.state.executer.inst.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.inst.fn_core)(self, 0);
                    self.mem.write(self.state.addr, val);
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
                if self.state.executer.inst.dst == Destination::Register {
                    let val = self.mem.read(addr);
                    (self.state.executer.inst.fn_core)(self, val);
                } else {
                    let val = (self.state.executer.inst.fn_core)(self, 0);
                    self.mem.write(addr, val);
                }
                if let Some(_) = low.checked_add(self.regs.y) {
                    self.exec_finished();
                } else {
                    self.state.executer.last_cycle += 1;
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
                (self.state.executer.inst.fn_core)(self, 0);
                self.exec_finished();
            },
            _ => unreachable!(),
        };
    }

    pub fn exec_push_stack(&mut self) {
        match self.state.counter {
            2 => (),
            3 => {
                (self.state.executer.inst.fn_core)(self, 0);
                self.exec_finished();
            },
            _ => unreachable!(),
        };
    }

    /// 注：この関数内で処理が完結する。
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
                self.exec_finished();
            },
            _ => unreachable!(),
        };
    }

    /// 注：この関数内で処理が完結する。
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
                self.exec_finished();
            },
            _ => unreachable!(),
        };
    }

    /// 注：この関数内で処理が完結する。
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
                self.exec_finished();
            },
            _ => unreachable!(),
        };
    }

    pub fn exec_accumulator(&mut self) {
        match self.state.counter {
            2 => {
                let result = (self.state.executer.inst.fn_core)(self, self.regs.a);
                // フラグは変更済みなので、ここでは代入するだけ
                self.regs.a = result;
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }

    /// Read-Modify-WriteなZeropageアドレッシング
    pub fn exec_zeropage_rmw(&mut self) {
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => {
                self.state.op_2 = self.mem.read(self.state.op_1 as u16);
            },
            4 => {
                self.state.op_2 = (self.state.executer.inst.fn_core)(self, self.state.op_2);
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
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_1 = self.state.op_1.wrapping_add(self.regs.x),
            4 => {
                self.state.addr = self.state.op_1 as u16;
                self.state.op_2 = self.mem.read(self.state.addr);
            },
            5 => {
                self.state.op_2 = (self.state.executer.inst.fn_core)(self, self.state.op_2);
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
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_2 = self.fetch(),
            4 => {
                let addr = make_addr(self.state.op_2, self.state.op_1);
                self.state.addr = addr;
                self.state.op_2 = self.mem.read(addr);
            },
            5 => {
                self.state.op_2 = (self.state.executer.inst.fn_core)(self, self.state.op_2);
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
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => {
                let low = self.state.op_1;
                let high = self.fetch();
                self.state.addr = make_addr(high, low).wrapping_add(self.regs.x as u16);
            },
            4 => (),
            5 => self.state.op_1 = self.mem.read(self.state.addr),
            6 => self.state.op_2 = (self.state.executer.inst.fn_core)(self, self.state.op_1),
            7 => {
                self.mem.write(self.state.addr, self.state.op_2);
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }

    /// 注：この関数内で処理が完結する。
    pub fn exec_absolute_jmp(&mut self) {
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => {
                let low = self.state.op_1;
                let high = self.fetch();
                self.regs.pc = make_addr(high, low);
                self.exec_finished();
            }
            _ => unreachable!(),
        }
    }

    /// 注：この関数内で処理が完結する。
    pub fn exec_indirect_jmp(&mut self) {
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => {
                let low = self.state.op_1;
                let high = self.fetch();
                self.state.addr = make_addr(high, low);
            },
            4 => self.state.op_1 = self.mem.read(self.state.addr),
            5 => {
                let low = self.state.op_1;
                let high = self.mem.read(self.state.addr.wrapping_add(1));
                self.regs.pc = make_addr(high, low);
                self.exec_finished();
            }
            _ => unreachable!(),
        }
    }

    /// 相対アドレッシングモード。このモードは分岐命令でのみ使われる。
    pub fn exec_relative(&mut self) {
        match self.state.counter {
            2 => {
                let offset = self.fetch();
                if (self.state.executer.inst.fn_core)(self, 0) == 0 {
                    // 分岐が発生しない場合はここで終わり
                    self.exec_finished();
                } else {
                    self.state.executer.last_cycle += 1;
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
                    // 分岐が発生して、かつ同じページ内へジャンプする場合は、例外の発生が1命令遅れる。
                    if self.int_requested.kind != IntType::None {
                        self.int_requested.is_force_delayed = true;
                    }
                    self.regs.pc = self.state.addr;
                    self.exec_finished();
                } else {
                    self.state.executer.last_cycle += 1;
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
