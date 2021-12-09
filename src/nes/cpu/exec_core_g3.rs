//! 命令実行のコア処理 (Group 3)

use super::{Cpu, Flags};

/*
Group 3 の全命令は以下の通り:
BRK JSR RTI RTS PHP PLP PHA PLA DEY TAY INY INX
CLC SEC CLI SEI TYA CLV CLD SED TXA TXS TAX TSX DEX NOP
*/

impl Cpu {

    //////////////////////////////////////////////
    /// JSR (absolute):
    /// スタックから、ステータスフラグと、PCをPullして設定する。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  - - - - - -
    //////////////////////////////////////////////
    pub fn jsr_action(&mut self, _: u8) -> u8 {
        log::debug!("[JSR]");
        // 何もしない
        0
    }

    //////////////////////////////////////////////
    /// RTI (implied/Stack):
    /// スタックから、ステータスフラグと、PCをPullして設定する。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  (スタックの内容によって上書き)
    //////////////////////////////////////////////
    pub fn rti_action(&mut self, _: u8) -> u8 {
        log::debug!("[RTI]");
        // 何もしない
        0
    }

    //////////////////////////////////////////////
    /// RTS (implied/Stack):
    /// 関数から呼び出し元に戻る。
    /// 具体的には、スタックからPCをPullし、その値+1 をPCに設定する。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  - - - - - -
    //////////////////////////////////////////////
    pub fn rts_action(&mut self, _: u8) -> u8 {
        log::debug!("[RTS]");
        // 何もしない
        0
    }

    //////////////////////////////////////////////
    /// PHP (Implied/Stack):
    /// ステータスレジスタの内容をスタックにPushし、スタックポインタを -1 する。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  - - - - - -
    //////////////////////////////////////////////
    pub fn php_action(&mut self, _: u8) -> u8 {
        log::debug!("[PHP]");
        self.push_stack(self.regs.p);
        0
    }

    //////////////////////////////////////////////
    /// PLP (Implied/Stack):
    /// スタックポインタを+1して、 スタックポインタの指す位置から値を取得し、
    /// ステータスレジスタに設定する。(ここに来た時点でスタックポインタは +1 されている)
    //////////////////////////////////////////////
    //  N Z C I D V
    //  (スタックの内容によって上書き)
    //////////////////////////////////////////////
    pub fn plp_action(&mut self, _: u8) -> u8 {
        log::debug!("[PLP]");
        self.regs.p = self.peek_stack();
        0
    }

    //////////////////////////////////////////////
    /// PHA (Implied/Stack):
    /// レジスタAの内容をスタックにPushし、スタックポインタを -1 する。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  - - - - - -
    //////////////////////////////////////////////
    pub fn pha_action(&mut self, _: u8) -> u8 {
        log::debug!("[PHA]");
        self.push_stack(self.regs.a);
        0
    }

    //////////////////////////////////////////////
    /// PLA (Implied/Stack):
    /// スタックポインタを+1して、 スタックポインタの指す位置から値を取得し、
    /// レジスタAに格納する。(ここに来た時点でスタックポインタは +1 されている)
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn pla_action(&mut self, _: u8) -> u8 {
        log::debug!("[PLA]");
        self.regs.a = self.peek_stack();
        self.regs.change_negative_by_value(self.regs.a);
        self.regs.change_zero_by_value(self.regs.a);
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
        self.regs.y = self.regs.y.wrapping_sub(1);
        self.regs.change_negative_by_value(self.regs.y);
        self.regs.change_zero_by_value(self.regs.y);
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
        self.regs.change_negative_by_value(self.regs.y);
        self.regs.change_zero_by_value(self.regs.y);
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
        self.regs.change_negative_by_value(self.regs.y);
        self.regs.change_zero_by_value(self.regs.y);
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
        self.regs.change_negative_by_value(self.regs.x);
        self.regs.change_zero_by_value(self.regs.x);
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
    /// TAX (implied):
    /// レジスタAをレジスタXにコピー。
    //////////////////////////////////////////////
    //  N Z C I D V
    //  + + - - - -
    //////////////////////////////////////////////
    pub fn tax_action(&mut self, _: u8) -> u8 {
        log::debug!("[TAX]");
        self.regs.x = self.regs.a;
        self.regs.change_negative_by_value(self.regs.x);
        self.regs.change_zero_by_value(self.regs.x);
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
        self.regs.change_negative_by_value(self.regs.x);
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
        self.regs.change_negative_by_value(self.regs.x);
        self.regs.change_zero_by_value(self.regs.x);
        0
    }

    //////////////////////////////////////////////
    /// NOP:
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