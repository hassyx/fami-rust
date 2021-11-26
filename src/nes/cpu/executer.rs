//! Instruction executer.

use super::Cpu;
use crate::nes::util::make_addr;

const BYTES_OF_PAGE: u16 = 0xFF;

// TODO: 割り込みのポーリングのタイミングは、本来は命令の最後から番目で行う。
// 現状は、命令が終了したタイミングでポーリングを解禁している。

impl Cpu {

    //////////////////////////////////////////////
    // ORA: レジスタAとメモリをORしてAに格納。
    //////////////////////////////////////////////
    
    pub fn ora_immediate(&mut self) {
        if self.state.counter == 2 {
            let operand = self.fetch();
            self.regs.a |= operand;
            self.exec_finished();
        }
    }

    pub fn ora_zeropage(&mut self) {
        match self.state.counter {
            2 => self.state.op_1 = self.fetch(),
            3 => {
                let val = self.mem.read(self.state.op_1 as u16);
                self.regs.a |= val;
                self.exec_finished();
            },
            _ => unreachable!(),
        }
    }

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

/*
     
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
