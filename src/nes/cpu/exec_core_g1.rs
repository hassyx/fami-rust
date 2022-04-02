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
        self.regs.cmp(self.regs.a, val);
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
        self.regs.a_sub(val);
        0
    }

    //////////////////////////////////////////////
    /// LSR (group 1, Read-Modify-Write):
    /// レジスタA、または指定されたアドレス上の値を右に1bitシフト。
    /// 空いたMSBには0を設定する。溢れたLSBはCarryに設定する。
    /// 先頭ビットが立っていればNegativeを、結果が0ならZeroを立てる。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  0 + + - - -
    //////////////////////////////////////////////
    pub fn lsr_action(&mut self, val: u8) -> u8 {
        // valを右シフトして、フラグを操作したあと戻り値として返す。
        let to_carry = val & Flags::CARRY.bits;
        let val = val >> 1;
        self.regs.p |= to_carry;
        self.regs.change_negative_by_value(val);
        self.regs.change_zero_by_value(val);
        val
    }

    //////////////////////////////////////////////
    /// ASL (group 1, Read-Modify-Write):
    /// レジスタA、または指定されたアドレス上の値を左に1bitシフト。
    /// 空いたLSBには0を設定する。溢れたMSBはCarryに設定する。
    /// 先頭ビットが立っていればNegativeを、結果が0ならZeroを立てる。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + + - - -
    //////////////////////////////////////////////
    pub fn asl_action(&mut self, val: u8) -> u8 {
        // valを左シフトして、フラグを操作したあと戻り値として返す。
        let to_carry = val & 0b1000_0000 >> 7;
        let val = val << 1;
        self.regs.p |= to_carry;
        self.regs.change_negative_by_value(val);
        self.regs.change_zero_by_value(val);
        val
    }

    //////////////////////////////////////////////
    /// ROR (group 1, Read-Modify-Write):
    /// レジスタA、または指定されたアドレス上の値を右に1bitローテート。
    /// 空いたMSBにはCarryを設定する。溢れたLSBはCarryに設定する。
    /// 先頭ビットが立っていればNegativeを、結果が0ならZeroを立てる。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + + - - -
    //////////////////////////////////////////////
    pub fn ror_action(&mut self, val: u8) -> u8 {
        // valを右ローテートして、フラグを操作したあと戻り値として返す。
        let from_carry = (self.regs.p & Flags::CARRY.bits) << 7;
        let to_carry = val & Flags::CARRY.bits;
        let val = (val >> 1) | from_carry;
        self.regs.p |= to_carry;
        self.regs.change_negative_by_value(val);
        self.regs.change_zero_by_value(val);
        val
    }

    //////////////////////////////////////////////
    /// ROL (group 1, Read-Modify-Write):
    /// レジスタA、または指定されたアドレス上の値を左に1bitローテート。
    /// 空いたLSBにはCarryを設定する。溢れたMSBはCarryに設定する。
    /// 先頭ビットが立っていればNegativeを、結果が0ならZeroを立てる。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + + - - -
    //////////////////////////////////////////////
    pub fn rol_action(&mut self, val: u8) -> u8 {
        // valを左ローテートして、フラグを操作したあと戻り値として返す。
        let from_carry = self.regs.p & Flags::CARRY.bits;
        let to_carry = val & 0b1000_0000 >> 7;
        let val = (val << 1) | from_carry;
        self.regs.p |= to_carry;
        self.regs.change_negative_by_value(val);
        self.regs.change_zero_by_value(val);
        val
    }

    //////////////////////////////////////////////
    /// DEC (group 1, Read-Modify-Write):
    /// メモリ上の値を1デクリメント。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn dec_action(&mut self, val: u8) -> u8 {
        let val = val.wrapping_sub(1);
        self.regs.change_negative_by_value(val);
        self.regs.change_zero_by_value(val);
        val
    }

    //////////////////////////////////////////////
    /// INC (group 1, Read-Modify-Write):
    /// メモリ上の値を1インクリメント。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn inc_action(&mut self, val: u8) -> u8 {
        let val = val.wrapping_add(1);
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
        self.regs.x_set(val);
        0
    }

    //////////////////////////////////////////////
    /// BIT (group 1):
    /// レジスタAとメモリ上の値(M)をAND。
    /// レジスタもメモリも汚さず、フラグだけ変動。
    /// 演算結果=0ならZeroが1。
    /// NegativeとOverflowは、演算結果ではなく、Mの値で変動する。
    /// M7がNegative、M6がOverflow。
    //////////////////////////////////////////////
    //  N  Z C I D V
    //  M7 + - - - M6
    //////////////////////////////////////////////
    pub fn bit_action(&mut self, val: u8) -> u8 {
        let new_val = self.regs.a & val;
        // 演算対象となるメモリ上の値によってフラグレジスタが変動
        let flags_nv = (val & !Flags::NEGATIVE.bits) | (val & !Flags::OVERFLOW.bits);
        // Zeroに限っては演算結果の方を見る
        let flags_z = ((new_val == 0) as u8) << 1;
        let flag_origin = self.regs.p & (!Flags::NEGATIVE.bits | !Flags::OVERFLOW.bits | !Flags::ZERO.bits);
        self.regs.p = flag_origin | flags_nv | flags_z;
        0
    }

    //////////////////////////////////////////////
    /// STY (group 1):
    /// レジスタYをメモリにストア。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  - - - - - -
    //////////////////////////////////////////////
    pub fn sty_action(&mut self, _: u8) -> u8 {
        self.regs.y
    }
    
    //////////////////////////////////////////////
    /// LDY (group 1):
    /// 値をレジスタYにロード。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn ldy_action(&mut self, val: u8) -> u8 {
        self.regs.y_set(val);
        0
    }

    //////////////////////////////////////////////
    /// CPY (group 1):
    /// レジスタYとメモリを比較(A - memory)し、
    /// 同じ値ならZreoをon、違うならOff。
    /// 結果のMSBが1ならNegativeをOn、0ならOff。
    /// Y >= memory ならCarryをOn、そうでなければOff。
    /// なお、レジスタYの内容には影響を与えない。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + + - - -
    //////////////////////////////////////////////
    pub fn cpy_action(&mut self, val: u8) -> u8 {
        self.regs.cmp(self.regs.y, val);
        0
    }

    //////////////////////////////////////////////
    /// CPX (group 1):
    /// レジスタXとメモリを比較(A - memory)し、
    /// 同じ値ならZreoをon、違うならOff。
    /// 結果のMSBが1ならNegativeをOn、0ならOff。
    /// X >= memory ならCarryをOn、そうでなければOff。
    /// なお、レジスタXの内容には影響を与えない。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + + - - -
    //////////////////////////////////////////////
    pub fn cpx_action(&mut self, val: u8) -> u8 {
        self.regs.cmp(self.regs.x, val);
        0
    }

    //////////////////////////////////////////////
    /// JMP (group 1):
    /// 指定されたアドレスへジャンプ。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  - - - - - -
    //////////////////////////////////////////////
    pub fn jmp_action(&mut self, _: u8) -> u8 {
        // 何もしない
        0
    }
    
}

