//! Instruction decoder.

use super::Cpu;

/// アドレッシングモード
/// TODO: matchで分岐する場合は、頻出するモードを先に置く。
#[derive(PartialEq)]
pub enum AddrMode {
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
}

impl Cpu {

    pub fn decode(&mut self) -> u8 {
        // 当面は非公式命令を検出した場合にpanicさせる。
        let opcode = self.fetch();
        if let Some(wait) = self.decode_tier1(opcode) {
            return wait;
        }
        if let Some(wait) = self.decode_tier2(opcode) {
            return wait;
        }
        if let Some(wait) = self.decode_tier3(opcode) {
            return wait;
        }

        self.panic_invalid_op(opcode);
    }

    fn panic_invalid_op(&mut self, opcode: u8) -> ! {
        panic!("\"{:#0X}\" is invalid opcode.", opcode);
    }

    /// OPコードの末尾2ビットを使った解析
    fn decode_tier1(&mut self, opcode: u8) -> Option<u8> {
        // "aaabbbcc" で分類
        // aaa,cc = OPコード,  bbb = アドレッシングモード
        let aaa = (opcode & 0b1110_0000) >> 4;
        let bbb = (opcode & 0b0001_1100) >> 2;
        let cc = opcode & 0b0000_0011;

        if cc == 0b01 {
            let addr_mode = self.decode_addr_tier1_01(bbb);
            if addr_mode == AddrMode::Invalid {
                self.panic_invalid_op(opcode);
            }

            match aaa {
                0b000 => {
                    // ORA
                    //self.ora(addr_mode)
                },
                0b001 => {},    // AND
                0b010 => {},    // EOR
                0b011 => {},    // ADC
                0b100 => {
                    // STA
                    // immediateなSTAは存在しない
                },    
                0b101 => {},    // LDA
                0b110 => {},    // CMP
                0b111 => {},    // SBC
                _ => self.panic_invalid_op(opcode),
            }
        } else if cc == 0b10 {
            // 注意：STXとLDXでは、IndexedZeroPage_X は Y を見る。
            // また、LDXでは、IndexedAbsolute_X は Y を見る。
            let addr_mode = self.decode_addr_tier1_10(opcode);
            if addr_mode == AddrMode::Invalid {
                self.panic_invalid_op(opcode);
            }

            match aaa {
                0b000 => {},    // ASL
                0b001 => {},    // ROL
                0b010 => {},    // LSR
                0b011 => {},    // ROR
                0b100 => {},    // STX
                0b101 => {},    // LDX
                0b110 => {},    // DEC
                0b111 => {},    // INC
                _ => self.panic_invalid_op(opcode),
            }
        } else if cc == 0b00 {
            let addr_mode = self.decode_addr_tier1_00(opcode);
            if addr_mode == AddrMode::Invalid {
                self.panic_invalid_op(opcode);
            }

            match aaa {
                0b001 => {},    //BIT
                0b010 => {},    //JMP
                0b011 => {},    //JMP (abs)
                0b100 => {},    //STY
                0b101 => {},    //LDY
                0b110 => {},    //CPY
                0b111 => {},    //CPX
                _ => self.panic_invalid_op(opcode),
            }
        } else if cc == 0b11 {
            self.panic_invalid_op(opcode);
        };

        // TODO: 消すこと
        None
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

    /// OPコードの末尾5ビットを使った解析
    fn decode_tier2(&mut self, opcode: u8) -> Option<u8> {
        // "xxy10000" は全て条件付きブランチ。
        // xx = OPコード, y = 比較に用いる値
        let op = (opcode & 0b1100_0000) >> 5;
        let val = (opcode & 0b0010_0000) >> 4;
        let tail = opcode & 0b0001_1111;

        // TODO: 具体的には以下の命令のいずれかだが、実際には「どの命令か」を
        // 知る必要はなく、デコードした結果を利用して演算すればよい。
        // ただし、デバッグ用に命令名を出力する必要がある。
        /*
            BPL	BMI	BVC	BVS	BCC	BCS	BNE	BEQ
            10	30	50	70	90	B0	D0	F0
        */

        if tail == 0b0001_0000 {
            match op {
                // check negative flag
                0b00 => {
                    // BPL or BMI
                },
                // check overflow flag
                0b01 => {
                    // BVC or BVS
                },
                // check carry flag
                0b10 => {
                    // BCC or BCS
                },
                // check zero flag
                0b11 => {
                    // BNE or BEQ
                },
                _ => self.panic_invalid_op(opcode),
            }
        };
        
        None
    }

    /// その他の1バイト命令をデコード
    fn decode_tier3(&mut self, opcode: u8) -> Option<u8> {
        // 注意：1バイト命令の次にはもう1バイトのパディング領域があるため、実際には2バイト長になる。
        match opcode {
            0x00 => {},     // BRK
            0x20 => {},     // JSR (abs)
            0x40 => {},     // RTI
            0x60 => {},     // RTS
            0x08 => {},     // PHP
            0x28 => {},     // PLP
            0x48 => {},     // PHA
            0x68 => {},     // PLA
            0x88 => {},     // DEY
            0xA8 => {},     // TAY
            0xC8 => {},     // INY
            0xE8 => {},     // INX
            0x18 => {},     // CLC
            0x38 => {},     // SEC
            0x58 => {},     // CLI
            0x78 => {},     // SEI
            0x98 => {},     // TYA
            0xB8 => {},     // CLV
            0xD8 => {},     // CLD
            0xF8 => {},     // SED
            0x8A => {},     // TXA
            0x9A => {},     // TXS
            0xAA => {},     // TAX
            0xBA => {},     // TSX
            0xCA => {},     // DEX
            0xEA => {},     // NOP
            _ => self.panic_invalid_op(opcode),
        }

        None
    }
}