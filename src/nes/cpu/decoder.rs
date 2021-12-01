//! Instruction decoder.

use super::Cpu;

/// 命令実行の実処理を担う関数ポインタの型
pub type PtrFnExec = fn(cpu: &mut Cpu);

/// アドレッシングモード
/// TODO: matchで分岐する場合は、頻出するモードを先に置く。
#[derive(Debug, PartialEq)]
enum AddrMode {
    /// 不正なアドレッシングモード。
    Invalid,
    /// Aレジスタに対して演算を行い、Aレジスタに格納する。
    Accumulator,
    /// オペランドの16bitの即値との演算。
    Immediate,
    /// オペランドで16bitのアドレスを指定し、参照先の8bitの値と演算を行う。
    Absolute,
    /// オペランドで8bitのアドレスを指定し、参照先の8bitの値と演算を行う。
    ZeroPage,
    /// オペランドで指定した16bitのアドレスに、レジスタXの値を足して、
    /// そのアドレスが指す8bitの値に対して演算を行う。
    IndexedAbsolute_X,
    /// オペランドで指定した16bitのアドレスに、レジスタYの値を足して、
    /// そのアドレスが指す8bitの値に対して演算を行う。
    /// 最終アドレスが16bitの最大値を超えた場合は、溢れた分を無視する。
    IndexedAbsolute_Y,
    /// オペランドで指定した8bitのアドレスに、レジスタX(一部の命令ではY)を加算して、
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
}

impl Cpu {

    pub fn panic_invalid_op(&mut self, opcode: u8) -> ! {
        panic!("\"{:#0X}\" is invalid opcode.", opcode);
    }

    /// OPコードをフェッチして命令種別を判定、実行を担う関数を返す。
    pub fn decode(&mut self) -> PtrFnExec {
        // 当面は非公式命令を検出した場合にpanicさせる。
        let opcode = self.fetch();
        if let Some(fn_exec) = self.decode_tier1(opcode) {
            return fn_exec
        }
        if let Some(fn_exec) = self.decode_tier2(opcode) {
            return fn_exec
        }
        if let Some(fn_exec) = self.decode_tier3(opcode) {
            return fn_exec;
        }

        self.panic_invalid_op(opcode);
    }

    fn get_ora_func(&mut self, addr_mode: AddrMode) -> PtrFnExec {
        match addr_mode {
            AddrMode::Immediate => Cpu::ora_absolute,
            AddrMode::ZeroPage => Cpu::ora_zeropage,
            AddrMode::IndexedZeroPage_X => Cpu::ora_indexed_zeroPage_x,
            AddrMode::Absolute => Cpu::ora_absolute,
            AddrMode::IndexedAbsolute_X => Cpu::ora_indexed_absolute_x,
            AddrMode::IndexedAbsolute_Y => Cpu::ora_indexed_absolute_y,
            AddrMode::IndexedIndirect_X => Cpu::ora_indexed_indirect_x,
            AddrMode::IndirectIndexed_Y => Cpu::ora_indirect_indexed_y,
            _ => panic!("ORA: {:?} is invalid addressing mode.", addr_mode),
        }
    }

    /// OPコードの末尾2ビットを使った解析
    fn decode_tier1(&mut self, opcode: u8) -> Option<PtrFnExec> {
        // "aaabbbcc" で分類
        // aaa,cc = OPコード,  bbb = アドレッシングモード
        let aaa = (opcode & 0b1110_0000) >> 4;
        let bbb = (opcode & 0b0001_1100) >> 2;
        let cc = opcode & 0b0000_0011;

        if cc == 0b01 {
            let addr_mode = self.decode_addr_tier1_01(bbb);
            if addr_mode == AddrMode::Invalid {
                return None
            }

            match aaa {
                0b000 => {
                    // ORA
                    return Some(self.get_ora_func(addr_mode))
                },
                0b001 => None,    // AND
                0b010 => None,    // EOR
                0b011 => None,    // ADC
                0b100 => None,    // STA (immediateなSTAは存在しない) 
                0b101 => None,    // LDA
                0b110 => None,    // CMP
                0b111 => None,    // SBC
                _ => None,
            }
        } else if cc == 0b10 {
            // 注意：STXとLDXでは、IndexedZeroPage_X は Y を見る。
            // また、LDXでは、IndexedAbsolute_X は Y を見る。
            let addr_mode = self.decode_addr_tier1_10(opcode);
            if addr_mode == AddrMode::Invalid {
                return None
            }

            match aaa {
                0b000 => None,    // ASL
                0b001 => None,    // ROL
                0b010 => None,    // LSR
                0b011 => None,    // ROR
                0b100 => None,    // STX
                0b101 => None,    // LDX
                0b110 => None,    // DEC
                0b111 => None,    // INC
                _ => None,
            }
        } else if cc == 0b00 {
            let addr_mode = self.decode_addr_tier1_00(opcode);
            if addr_mode == AddrMode::Invalid {
                return None
            }

            match aaa {
                0b001 => None,    //BIT
                0b010 => None,    //JMP
                0b011 => None,    //JMP (abs)
                0b100 => None,    //STY
                0b101 => None,    //LDY
                0b110 => None,    //CPY
                0b111 => None,    //CPX
                _ => None,
            }
        } else if cc == 0b11 {
            // 末尾が11の命令は存在しない
            self.panic_invalid_op(opcode);
        } else {
            None
        }
    }

    /// "aaabbbcc" 形式の命令で cc=01 の場合。
    /// "bbb" を利用したアドレッシングモードのデコード。
    fn decode_addr_tier1_01(&mut self, bbb: u8) -> AddrMode {
        match bbb {
            0b000 => AddrMode::IndexedIndirect_X,
            0b001 => AddrMode::ZeroPage,
            0b010 => AddrMode::Immediate,
            0b011 => AddrMode::Absolute,
            0b100 => AddrMode::IndirectIndexed_Y,
            0b101 => AddrMode::IndexedZeroPage_X,
            0b110 => AddrMode::IndexedAbsolute_Y,
            0b111 => AddrMode::IndexedAbsolute_X,
            _ => AddrMode::Invalid,
        }
    }

    /// "aaabbbcc" 形式の命令で cc=10 の場合。
    /// "bbb" を利用したアドレッシングモードのデコード。
    fn decode_addr_tier1_10(&mut self, bbb: u8) -> AddrMode {
        match bbb {
            0b000 => AddrMode::Immediate,
            0b001 => AddrMode::ZeroPage,
            0b010 => AddrMode::Accumulator,
            0b011 => AddrMode::Absolute,
            0b101 => AddrMode::IndexedIndirect_X,
            0b111 => AddrMode::IndexedAbsolute_X,
            _ => AddrMode::Invalid,
        }
    }

    /*
        全命令：
        ORA AND EOR ADC STA LDA CMP SBC
        ASL ROL LSR ROR STX LDX DEC INC
    */
    /// "aaabbbcc" 形式の命令で cc=00 の場合。
    /// "bbb" を利用したアドレッシングモードのデコード。
    fn decode_addr_tier1_00(&mut self, bbb: u8) -> AddrMode {
        match bbb {
            0b000 => AddrMode::Immediate,
            0b001 => AddrMode::ZeroPage,
            0b011 => AddrMode::Absolute,
            0b101 => AddrMode::IndexedIndirect_X,
            0b111 => AddrMode::IndexedAbsolute_X,
            _ => AddrMode::Invalid,
        }
    }

    /*
        全命令： BIT JMP JMP STY LDY CPY CPX
    */
    /// OPコードの末尾5ビットを使った解析
    fn decode_tier2(&mut self, opcode: u8) -> Option<PtrFnExec> {
        // "xxy10000" は全て条件付きブランチ。
        // xx = OPコード, y = 比較に用いる値
        let op = (opcode & 0b1100_0000) >> 5;
        let val = (opcode & 0b0010_0000) >> 4;
        let tail = opcode & 0b0001_1111;

        if tail != 0b0001_0000 {
            None
        } else {
            match op {
                // check negative flag
                0b00 => {
                    // BPL or BMI
                    None
                },
                // check overflow flag
                0b01 => {
                    // BVC or BVS
                    None
                },
                // check carry flag
                0b10 => {
                    // BCC or BCS
                    None
                },
                // check zero flag
                0b11 => {
                    // BNE or BEQ
                    None
                },
                _ => None,
            }
        }
    }

    /*
        全命令：
        BRK JSR abs RTI RTS PHP PLP PHA PLA DEY TAY INY INX
        CLC SEC CLI SEI TYA CLV CLD SED TXA TXS TAX TSX DEX NOP
    */
    /// その他の1バイト命令をデコード
    fn decode_tier3(&mut self, opcode: u8) -> Option<PtrFnExec> {
        match opcode {
            0x00 => None,     // BRK
            0x20 => None,     // JSR (abs)
            0x40 => None,     // RTI
            0x60 => None,     // RTS
            0x08 => None,     // PHP
            0x28 => None,     // PLP
            0x48 => None,     // PHA
            0x68 => None,     // PLA
            0x88 => None,     // DEY
            0xA8 => None,     // TAY
            0xC8 => None,     // INY
            0xE8 => None,     // INX
            0x18 => None,     // CLC
            0x38 => None,     // SEC
            0x58 => None,     // CLI
            0x78 => {   // SEI
                Some(Cpu::sei)
            },
            0x98 => None,     // TYA
            0xB8 => None,     // CLV
            0xD8 => None,     // CLD
            0xF8 => None,     // SED
            0x8A => None,     // TXA
            0x9A => None,     // TXS
            0xAA => None,     // TAX
            0xBA => None,     // TSX
            0xCA => None,     // DEX
            0xEA => None,     // NOP
            _ => None,
        }
    }
}