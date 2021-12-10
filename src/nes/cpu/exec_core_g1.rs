//! 命令実行のコア処理 (Group 1)

use super::{Cpu, Flags};

/*
Group 1 の全命令は以下の通り:
ORA AND EOR ADC STA LDA CMP SBC
ASL ROL LSR ROR STX LDX DEC INC
*/

impl Cpu {

    //////////////////////////////////////////////
    /// ORA (group 1):
    /// レジスタAとメモリをORしてAに格納。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn ora_action(&mut self, val: u8) -> u8 {
        log::debug!("[ORA]");
        self.regs.a_set(self.regs.a | val);
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
        self.regs.a_set(self.regs.a & val);
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
        self.regs.a_set(self.regs.a ^ val);
        0
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
    /// LDA (group 1):
    /// 値をレジスタAにロード。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn lda_action(&mut self, val: u8) -> u8 {
        log::debug!("[LDA]");
        self.regs.a_set(val);
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
    /// ROR (group 1):
    /// レジスタA、または指定されたアドレス上の値を右に1bitローテート。
    /// 空いたMSBにはCarryを設定する。溢れたLSBはCarryに設定する。
    /// 先頭ビットが立っていればNegativeを、結果が0ならZeroを立てる。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + + - - -
    //////////////////////////////////////////////
    pub fn ror_action(&mut self, val: u8) -> u8 {
        log::debug!("[ROR]");
        // valを右ローテートして、フラグを操作したあと戻り値として返す。
        let from_carry = (self.regs.p & Flags::CARRY.bits) << 7;
        let to_carry = val & !Flags::CARRY.bits;
        let val = (val >> 1) | from_carry;
        self.regs.p |= to_carry;
        self.regs.change_negative_by_value(val);
        self.regs.change_zero_by_value(val);
        val
    }

    //////////////////////////////////////////////
    /// STX (group 1):
    /// レジスタXをメモリにストア。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  - - - - - -
    //////////////////////////////////////////////
    pub fn stx_action(&mut self, _: u8) -> u8 {
        log::debug!("[STX]");
        self.regs.x
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
        self.regs.x_set(val);
        0
    }

}

