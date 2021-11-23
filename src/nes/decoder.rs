//! Instruction decoder.

use crate::nes::cpu::Cpu;

pub struct Decoder {

}

impl Decoder {

    fn panic_invalid_op(opcode: u8) -> ! {
        panic!("\"{:0x}\" is undocumented opcode.", opcode);
    }

    pub fn decode(cpu: &mut Cpu) -> u8 {
        // 当面は非公式命令を検出した場合にpanicさせる。
        let opcode = cpu.fetch();
        if let Some(wait) = Decoder::decode_by_last_2bit(opcode) {
            return wait;
        }
        if let Some(wait) = Decoder::decode_by_last_5bit(opcode) {
            return wait;
        }
        if let Some(wait) = Decoder::decode_remains(opcode) {
            return wait;
        }

        Decoder::panic_invalid_op(opcode);
    }

        /// OPコードの末尾2ビットを使った解析
    fn decode_by_last_2bit(opcode: u8) -> Option<u8> {
        // "aaabbbcc" で分類
        // aaa,cc = OPコード,  bbb = アドレッシングモード
        let aaa = (opcode & 0b1110_0000) >> 4;
        let addr_mode = (opcode & 0b0001_1100) >> 2;
        let cc = opcode & 0b0000_0011;

        if cc == 0b01 {
            match aaa {
                // アドレッシングモードに応じた実装を追加
                /*
                    bbb	addressing mode
                    000	(zero page,X)
                    001	zero page
                    010	#immediate
                    011	absolute
                    100	(zero page),Y
                    101	zero page,X
                    110	absolute,Y
                    111	absolute,X
                */
                0b000 => {},    // ORA
                0b001 => {},    // AND
                0b010 => {},    // EOR
                0b011 => {},    // ADC
                0b100 => {},    // STA
                0b101 => {},    // LDA
                0b110 => {},    // CMP
                0b111 => {},    // SBC
                _ => Decoder::panic_invalid_op(opcode),
            }
        } else if cc == 0b10 {
            match aaa {
                // アドレッシングモードに応じた実装を追加
                /*
                    bbb	addressing mode
                    000	#immediate
                    001	zero page
                    010	accumulator
                    011	absolute
                    101	zero page,X
                    111	absolute,X
                */
                0b000 => {},    // ASL
                0b001 => {},    // ROL
                0b010 => {},    // LSR
                0b011 => {},    // ROR
                0b100 => {},    // STX
                0b101 => {},    // LDX
                0b110 => {},    // DEC
                0b111 => {},    // INC
                _ => Decoder::panic_invalid_op(opcode),
            }
        } else if cc == 0b00 {
            match aaa {
                // アドレッシングモードに応じた実装を追加
                /*
                    bbb	addressing mode
                    000	#immediate
                    001	zero page
                    011	absolute
                    101	zero page,X
                    111	absolute,X
                */
                0b001 => {},    //BIT
                0b010 => {},    //JMP
                0b011 => {},    //JMP (abs)
                0b100 => {},    //STY
                0b101 => {},    //LDY
                0b110 => {},    //CPY
                0b111 => {},    //CPX
                _ => Decoder::panic_invalid_op(opcode),
            }
        } else if cc == 0b11 {
            Decoder::panic_invalid_op(opcode);
        };

        // TODO: 消すこと
        None
    }

    /// OPコードの末尾5ビットを使った解析
    fn decode_by_last_5bit(opcode: u8) -> Option<u8> {
        // "xxy10000" は全て条件付きブランチ。
        // xx = OPコード, y = 比較に用いる値
        let xx = (opcode & 0b1100_0000) >> 5;
        let y = (opcode & 0b0010_0000) >> 4;
        let tail = opcode & 0b0001_1111;

        // TODO: 具体的には以下の命令にいずれかだが、実際には
        // 「どの命令か」を知る必要はなく、ビットに記されるままに実行すればいい。
        // ただし、デバッグ用に命令名を表示する必要がある。
        /*
            BPL	BMI	BVC	BVS	BCC	BCS	BNE	BEQ
            10	30	50	70	90	B0	D0	F0
        */

        if tail == 0b0001_0000 {
            match xx {
                0b00 => {},     // negative
                0b01 => {},     // overflow
                0b10 => {},     // carry
                0b11 => {},     // zero
                _ => Decoder::panic_invalid_op(opcode),
            }
        } else {
            
        };
        
        None
    }

    fn decode_remains(opcode: u8) -> Option<u8> {
        // 注意：1バイト命令はもう1バイトのパディングがあるため、2バイト長になる。
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
            _ => Decoder::panic_invalid_op(opcode),
        }

        None
    }
}