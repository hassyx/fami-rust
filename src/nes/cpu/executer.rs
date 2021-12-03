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

    pub fn exec_indexed_zeroPage_x(&mut self) {
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
        self.regs.a |= val;
        0
    }

    //////////////////////////////////////////////
    /// LDA: 値をレジスタAにロード。
    //////////////////////////////////////////////
    pub fn lda_action(&mut self, val: u8) -> u8 {
        self.regs.a = val;
        0
    }

    //////////////////////////////////////////////
    /// STA: レジスタAの内容をメモリに書き込む。
    //////////////////////////////////////////////
    pub fn sta_action(&mut self, val: u8) -> u8 {
        self.regs.a = val;
        0
    }

    //////////////////////////////////////////////
    /// SEI: 割り込み禁止フラグを立てる。
    //////////////////////////////////////////////
    pub fn sei_action(&mut self, _: u8) -> u8 {
        self.regs.flags_on(Flags::INT_DISABLE);
        0
    }

    //////////////////////////////////////////////
    /// CLD: Overflowフラグをクリア。
    //////////////////////////////////////////////
    pub fn cld_action(&mut self, _: u8) -> u8 {
        self.regs.flags_off(Flags::OVERFLOW);
        0
    }

    /*
    pub fn ora_indexed_zeroPage_x(&mut self) {
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_1 = self.state.op_1.wrapping_add(self.regs.x),
            4 => {
                let addr = self.state.op_1 as u16;
                let val = self.mem.read(addr);
                self.regs.a |= val;
                self.exec_finished();
            }
            _ => unreachable!(),
        }
    }

    pub fn ora_absolute(&mut self) {
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_2 = self.fetch(),
            4 => {
                let addr = make_addr(self.state.op_2, self.state.op_1);
                let val = self.mem.read(addr);
                self.regs.a |= val;
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }

    pub fn ora_indexed_absolute_x(&mut self) {
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_2 = self.fetch(),
            4 => {
                let low = self.state.op_1;
                let high = self.state.op_2;
                let addr = make_addr(high, low).wrapping_add(self.regs.x as u16);
                let val = self.mem.read(addr);
                self.regs.a |= val;
                if let Some(_) = low.checked_add(self.regs.x) {
                    self.exec_finished()
                }
            },
            5 => self.exec_finished(),
            _ => unreachable!(),
        }
    }

    pub fn ora_indexed_absolute_y(&mut self) {
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => self.state.op_2 = self.fetch(),
            4 => {
                let low = self.state.op_1;
                let high = self.state.op_2;
                let addr = make_addr(high, low).wrapping_add(self.regs.y as u16);
                let val = self.mem.read(addr);
                self.regs.a |= val;
                if let Some(_) = low.checked_add(self.regs.y) {
                    self.exec_finished();
                }
            },
            5 => self.exec_finished(),
            _ => unreachable!(),
        }
    }

    pub fn ora_indexed_indirect_x(&mut self) {
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
                let val = self.mem.read(addr);
                self.regs.a |= val;
                self.exec_finished();
            }
            _ => unreachable!(),
        }
    }

    pub fn ora_indirect_indexed_y(&mut self) {
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
                let val = self.mem.read(addr);
                self.regs.a |= val;
                if let Some(_) = low.checked_add(self.regs.y) {
                    self.exec_finished();
                }
            }
            6 => self.exec_finished(),
            _ => unreachable!(),
        }
    }
    */


/*
    *
    add 1 to cycles if page boundary is crossed
    **
    add 1 to cycles if branch occurs on same page
    add 2 to cycles if branch occurs to different page
*/


    // ここに来た時点でOPコードのフェッチが済んでいる。
    // 命令の実行完了までに要するクロックサイクル数は、
    // OPコードのフェッチも含めた全体のクロック数を返す。
    /*
    /// ORA: レジスタAとメモリをORしてAに格納。
    pub fn ora(&mut self, addr_mode: AddrMode) -> u8 {
        let operand = self.fetch();
        let clk_cnt: u8;

        let val = match addr_mode {
            AddrMode::Immediate => {
                clk_cnt = 2;
                operand
            },
            AddrMode::ZeroPage => {
                clk_cnt = 3;
                self.mem.read(operand as u16)
            },
            AddrMode::IndexedZeroPage_X => {
                clk_cnt = 4;
                let addr = self.regs.x.wrapping_add(operand) as u16;
                self.mem.read(addr)
            },
            AddrMode::Absolute => {
                clk_cnt = 4;
                let addr = ((self.fetch() as u16) << 8) | operand as u16;
                self.mem.read(addr)
            },
            /*
            AddrMode::IndexedAbsolute_X => {
                
                if 
            },
            */

            _ => panic!("!!!!! dead")
        };

        self.regs.a |= val;
        clk_cnt
    }
    */




/*
    /// 不正なアドレッシングモード。
    Invalid,
    /// Aレジスタに対して演算を行い、Aレジスタに格納する。
    Accumulator,
    /// オペランドの16bitの即値との演算。
    Immediate,
    /// オペランドで16bitのアドレスを指定し、参照先の8bitの値と演算を行う。
    Absolute,
    /// オペランドで16bitのアドレス(ただし0-255の範囲)を指定し、参照先の8bitの値と演算を行う。
    ZeroPage,
    /// オペランドで指定した16bitのアドレスに、レジスタXの値を足して、
    /// そのアドレスが指す8bitの値に対して演算を行う。
    IndexedAbsolute_X,
    /// オペランドで指定した16bitのアドレスに、レジスタYの値を足して、
    /// そのアドレスが指す8bitの値に対して演算を行う。
    IndexedAbsolute_Y,
    /// オペランドで指定した16bitのアドレス(ただし範囲は0-255)に、レジスタX(一部の命令ではY)を加算して、
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
*/
}
