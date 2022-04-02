use core::fmt;
use std::fmt::Display;

use super::executer::{FnExec, FnCore};
use super::is_template::*;
use super::is_core::*;

/// アドレッシングモード
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AddrMode {
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
    /// 最終アドレスが16bitの最大値を超えた場合は、溢れた分を無視する。
    IndexedAbsoluteX,
    /// オペランドで指定した16bitのアドレスに、レジスタYの値を足して、
    /// そのアドレスが指す8bitの値に対して演算を行う。
    /// 最終アドレスが16bitの最大値を超えた場合は、溢れた分を無視する。
    IndexedAbsoluteY,
    /// オペランドで指定した8bitのアドレスに、レジスタX(一部の命令ではY)を加算して、
    /// そのアドレスが指す8bitの値に対して演算を行う。
    /// 算出したアドレスがゼロページ(0-255)を超過する、しないに関わらず、常に下位8bitの値しか見ない。
    IndexedZeroPageX,
    /// オペランドで指定した8bitの値に、レジスタXの値を足して、ゼロページ内のアドレスを得る。
    /// 次に、このアドレスの指す8bitを下位アドレス、アドレス+1 の指す内容を上位8bitとして、
    /// 16bitの最終アドレスを得る。この最終アドレスの指す先の、8bitの値に対して操作を行う。
    /// なお、1段階目と2段階目で算出したアドレスが8bitを越える、越えないに関わらず、常に下位の8bitのみを見る。
    IndexedIndirectX,
    /// オペランドで指定した8bitのアドレスを下位アドレス、アドレス+1 の指す内容を上位8bitとして、
    /// 16bitのアドレスを得る。このアドレスに、レジスタYの値を足して、最終アドレスを得る。
    /// 最終アドレスの指す先の8bitの値に対して操作を行う。
    /// なお、1段階目と2段階目で算出したアドレスが8bitを越える、越えないに関わらず、常に下位の8bitのみを見る。
    IndirectIndexedY,
    /// JMPでのみ使用。オペランドで指定した16bitのアドレスを下位8bit、
    /// そのアドレス+1 の指す内容を上位8bitとして、16bitのアドレスを得る。
    Indirect,
    /// 比較命令でのみ使用。現在のPCに8bitのオペランドを加算し、そのアドレスにジャンプする。
    /// なお、オペランドは符号ありの整数値として扱われる。
    Relative,
    /// 実行アドレスを必要としない命令。
    Implied,
}

#[derive(PartialEq, Eq, Clone, Copy)]
/// 最終的な演算結果を、レジスタに書き込むのか、それともメモリに書き込むのか。
pub enum Destination {
    /// レジスタに書き込む。NOPのような書き込み対象が存在しない命令や、
    /// レジスタ・メモリのどちらにも書き込む命令も、こちらに分類する。
    Register,
    /// メモリへ書き込む。
    Memory,
}

pub struct Instruction {
    pub core_name: &'static str,
    pub template_name: &'static str,
    pub fn_exec: FnExec,
    pub fn_core: FnCore,
    pub dst: Destination,
    pub min_clock: u8,
    pub addr_mode: AddrMode,
}

macro_rules! new_instruction{
    ($template:expr, $core:expr) => {
        Instruction {
            core_name: $core.name,
            template_name: $template.name,
            fn_exec: $template.fn_exec,
            fn_core: $core.fn_core,
            dst: $core.dst,
            min_clock: $template.min_clock,
            addr_mode: $template.addr_mode,
        }
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.core_name, self.template_name)
    }
}

pub const INSTRUCTION_SET: [Option<&Instruction>; 256] = [
    None, // 0x00:BRK (BRKは割り込みとして処理するので不要)
    Some(&ORA_INDEXED_INDIRECT_X), // 0x01:ORA X,ind
    None, // 0x02:---
    None, // 0x03:---
    None, // 0x04:---
    Some(&ORA_ZEROPAGE), // 0x05:ORA zpg
    Some(&ASL_ZEROPAGE), // 0x06:ASL zpg
    None, // 0x07:---
    Some(&PHP), // 0x08:PHP impl
    Some(&ORA_IMMEDIATE), // 0x09:ORA #
    Some(&ASL_ACCUMULATOR), // 0x0A:ASL A
    None, // 0x0B:---
    None, // 0x0C:---
    Some(&ORA_ABSOLUTE), // 0x0D:ORA abs
    Some(&ASL_ABSOLUTE), // 0x0E:ASL abs
    None, // 0x0F:---
    Some(&BPL), // 0x10:BPL rel
    Some(&ORA_INDIRECT_INDEXED_Y), // 0x11:ORA ind,Y
    None, // 0x12:---
    None, // 0x13:---
    None, // 0x14:---
    Some(&ORA_INDEXED_ZEROPAGE_X), // 0x15:ORA zpg,X
    Some(&ASL_INDEXED_ZEROPAGE_X), // 0x16:ASL zpg,X
    None, // 0x17:---
    Some(&CLC), // 0x18:CLC impl
    Some(&ORA_INDEXED_ABSOLUTE_Y), // 0x19:ORA abs,Y
    None, // 0x1A:---
    None, // 0x1B:---
    None, // 0x1C:---
    Some(&ORA_INDEXED_ABSOLUTE_X), // 0x1D:ORA abs,X
    Some(&ASL_INDEXED_ABSOLUTE_X), // 0x1E:ASL abs,X
    None, // 0x1F:---
    Some(&JSR), // 0x20:JSR abs
    Some(&AND_INDEXED_INDIRECT_X), // 0x21:AND X,ind
    None, // 0x22:---
    None, // 0x23:---
    Some(&BIT_ZEROPAGE), // 0x24:BIT zpg
    Some(&AND_ZEROPAGE), // 0x25:AND zpg
    Some(&ROL_ZEROPAGE), // 0x26:ROL zpg
    None, // 0x27:---
    Some(&PLP), // 0x28:PLP impl
    Some(&AND_IMMEDIATE), // 0x29:AND #
    Some(&ROL_ACCUMULATOR), // 0x2A:ROL A
    None, // 0x2B:---
    Some(&BIT_ABSOLUTE), // 0x2C:BIT abs
    Some(&AND_ABSOLUTE), // 0x2D:AND abs
    Some(&ROL_ABSOLUTE), // 0x2E:ROL abs
    None, // 0x2F:---
    Some(&BMI), // 0x30:BMI rel
    Some(&AND_INDIRECT_INDEXED_Y), // 0x31:AND ind,Y
    None, // 0x32:---
    None, // 0x33:---
    None, // 0x34:---
    Some(&AND_INDEXED_ZEROPAGE_X), // 0x35:AND zpg,X
    Some(&ROL_INDEXED_ZEROPAGE_X), // 0x36:ROL zpg,X
    None, // 0x37:---
    Some(&SEC), // 0x38:SEC impl
    Some(&AND_INDEXED_ABSOLUTE_Y), // 0x39:AND abs,Y
    None, // 0x3A:---
    None, // 0x3B:---
    None, // 0x3C:---
    Some(&AND_INDEXED_ABSOLUTE_X), // 0x3D:AND abs,X
    Some(&ROL_INDEXED_ABSOLUTE_X), // 0x3E:ROL abs,X
    None, // 0x3F:---
    Some(&RTI), // 0x40:RTI impl
    Some(&EOR_INDEXED_INDIRECT_X), // 0x41:EOR X,ind
    None, // 0x42:---
    None, // 0x43:---
    None, // 0x44:---
    Some(&EOR_ZEROPAGE), // 0x45:EOR zpg
    Some(&LSR_ZEROPAGE), // 0x46:LSR zpg
    None, // 0x47:---
    Some(&PHA), // 0x48:PHA impl
    Some(&EOR_IMMEDIATE), // 0x49:EOR #
    Some(&LSR_ACCUMULATOR), // 0x4A:LSR A
    None, // 0x4B:---
    Some(&JMP_ABSOLUTE), // 0x4C:JMP abs
    Some(&EOR_ABSOLUTE), // 0x4D:EOR abs
    Some(&LSR_ABSOLUTE), // 0x4E:LSR abs
    None, // 0x4F:---
    Some(&BVC), // 0x50:BVC rel
    Some(&EOR_INDIRECT_INDEXED_Y), // 0x51:EOR ind,Y
    None, // 0x52:---
    None, // 0x53:---
    None, // 0x54:---
    Some(&EOR_INDEXED_ZEROPAGE_X), // 0x55:EOR zpg,X
    Some(&LSR_INDEXED_ZEROPAGE_X), // 0x56:LSR zpg,X
    None, // 0x57:---
    Some(&CLI), // 0x58:CLI impl
    Some(&EOR_INDEXED_ABSOLUTE_Y), // 0x59:EOR abs,Y
    None, // 0x5A:---
    None, // 0x5B:---
    None, // 0x5C:---
    Some(&EOR_INDEXED_ABSOLUTE_X), // 0x5D:EOR abs,X
    Some(&LSR_INDEXED_ABSOLUTE_X), // 0x5E:LSR abs,X
    None, // 0x5F:---
    Some(&RTS), // 0x60:RTS impl
    Some(&ADC_INDEXED_INDIRECT_X), // 0x61:ADC X,ind
    None, // 0x62:---
    None, // 0x63:---
    None, // 0x64:---
    Some(&ADC_ZEROPAGE), // 0x65:ADC zpg
    Some(&ROR_ZEROPAGE), // 0x66:ROR zpg
    None, // 0x67:---
    Some(&PLA), // 0x68:PLA impl
    Some(&ADC_IMMEDIATE), // 0x69:ADC #
    Some(&ROR_ACCUMULATOR), // 0x6A:ROR A
    None, // 0x6B:---
    Some(&JMP_INDIRECT), // 0x6C:JMP ind
    Some(&ADC_ABSOLUTE), // 0x6D:ADC abs
    Some(&ROR_ABSOLUTE), // 0x6E:ROR abs
    None, // 0x6F:---
    Some(&BVS), // 0x70:BVS rel
    Some(&ADC_INDIRECT_INDEXED_Y), // 0x71:ADC ind,Y
    None, // 0x72:---
    None, // 0x73:---
    None, // 0x74:---
    Some(&ADC_INDEXED_ZEROPAGE_X), // 0x75:ADC zpg,X
    Some(&ROR_INDEXED_ZEROPAGE_X), // 0x76:ROR zpg,X
    None, // 0x77:---
    Some(&SEI), // 0x78:SEI impl
    Some(&ADC_INDEXED_ABSOLUTE_Y), // 0x79:ADC abs,Y
    None, // 0x7A:---
    None, // 0x7B:---
    None, // 0x7C:---
    Some(&ADC_INDEXED_ABSOLUTE_X), // 0x7D:ADC abs,X
    Some(&ROR_INDEXED_ABSOLUTE_X), // 0x7E:ROR abs,X
    None, // 0x7F:---
    None, // 0x80:---
    Some(&STA_INDEXED_INDIRECT_X), // 0x81:STA X,ind
    None, // 0x82:---
    None, // 0x83:---
    Some(&STY_ZEROPAGE), // 0x84:STY zpg
    Some(&STA_ZEROPAGE), // 0x85:STA zpg
    Some(&STX_ZEROPAGE), // 0x86:STX zpg
    None, // 0x87:---
    Some(&DEY), // 0x88:DEY impl
    None, // 0x89:---
    Some(&TXA), // 0x8A:TXA impl
    None, // 0x8B:---
    Some(&STY_ABSOLUTE), // 0x8C:STY abs
    Some(&STA_ABSOLUTE), // 0x8D:STA abs
    Some(&STX_ABSOLUTE), // 0x8E:STX abs
    None, // 0x8F:---
    Some(&BCC), // 0x90:BCC rel
    Some(&STA_INDIRECT_INDEXED_Y), // 0x91:STA ind,Y
    None, // 0x92:---
    None, // 0x93:---
    Some(&STY_INDEXED_ZEROPAGE_X), // 0x94:STY zpg,X
    Some(&STA_INDEXED_ZEROPAGE_X), // 0x95:STA zpg,X
    Some(&STX_INDEXED_ZEROPAGE_Y), // 0x96:STX zpg,Y
    None, // 0x97:---
    Some(&TYA), // 0x98:TYA impl
    Some(&STA_INDEXED_ABSOLUTE_Y), // 0x99:STA abs,Y
    Some(&TXS), // 0x9A:TXS impl
    None, // 0x9B:---
    None, // 0x9C:---
    Some(&STA_INDEXED_ABSOLUTE_X), // 0x9D:STA abs,X
    None, // 0x9E:---
    None, // 0x9F:---
    Some(&LDY_IMMEDIATE), // 0xA0:LDY #
    Some(&LDA_INDEXED_INDIRECT_X), // 0xA1:LDA X,ind
    Some(&LDX_IMMEDIATE), // 0xA2:LDX #
    None, // 0xA3:---
    Some(&LDY_ZEROPAGE), // 0xA4:LDY zpg
    Some(&LDA_ZEROPAGE), // 0xA5:LDA zpg
    Some(&LDX_ZEROPAGE), // 0xA6:LDX zpg
    None, // 0xA7:---
    Some(&TAY), // 0xA8:TAY impl
    Some(&LDA_IMMEDIATE), // 0xA9:LDA #
    Some(&TAX), // 0xAA:TAX impl
    None, // 0xAB:---
    Some(&LDY_ABSOLUTE), // 0xAC:LDY abs
    Some(&LDA_ABSOLUTE), // 0xAD:LDA abs
    Some(&LDX_ABSOLUTE), // 0xAE:LDX abs
    None, // 0xAF:---
    Some(&BCS), // 0xB0:BCS rel
    Some(&LDA_INDIRECT_INDEXED_Y), // 0xB1:LDA ind,Y
    None, // 0xB2:---
    None, // 0xB3:---
    Some(&LDY_INDEXED_ZEROPAGE_X), // 0xB4:LDY zpg,X
    Some(&LDA_INDEXED_ZEROPAGE_X), // 0xB5:LDA zpg,X
    Some(&LDX_INDEXED_ZEROPAGE_Y), // 0xB6:LDX zpg,Y
    None, // 0xB7:---
    Some(&CLV), // 0xB8:CLV impl
    Some(&LDA_INDEXED_ABSOLUTE_Y), // 0xB9:LDA abs,Y
    Some(&TSX), // 0xBA:TSX impl
    None, // 0xBB:---
    Some(&LDY_INDEXED_ABSOLUTE_X), // 0xBC:LDY abs,X
    Some(&LDA_INDEXED_ABSOLUTE_X), // 0xBD:LDA abs,X
    Some(&LDX_INDEXED_ABSOLUTE_Y), // 0xBE:LDX abs,Y
    None, // 0xBF:---
    Some(&CPY_IMMEDIATE), // 0xC0:CPY #
    Some(&CMP_INDEXED_INDIRECT_X), // 0xC1:CMP X,ind
    None, // 0xC2:---
    None, // 0xC3:---
    Some(&CPY_ZEROPAGE), // 0xC4:CPY zpg
    Some(&CMP_ZEROPAGE), // 0xC5:CMP zpg
    Some(&DEC_ZEROPAGE), // 0xC6:DEC zpg
    None, // 0xC7:---
    Some(&INY), // 0xC8:INY impl
    Some(&CMP_IMMEDIATE), // 0xC9:CMP #
    Some(&DEX), // 0xCA:DEX impl
    None, // 0xCB:---
    Some(&CPY_ABSOLUTE), // 0xCC:CPY abs
    Some(&CMP_ABSOLUTE), // 0xCD:CMP abs
    Some(&DEC_ABSOLUTE), // 0xCE:DEC abs
    None, // 0xCF:---
    Some(&BNE), // 0xD0:BNE rel
    Some(&CMP_INDIRECT_INDEXED_Y), // 0xD1:CMP ind,Y
    None, // 0xD2:---
    None, // 0xD3:---
    None, // 0xD4:---
    Some(&CMP_INDEXED_ZEROPAGE_X), // 0xD5:CMP zpg,X
    Some(&DEC_INDEXED_ZEROPAGE_X), // 0xD6:DEC zpg,X
    None, // 0xD7:---
    Some(&CLD), // 0xD8:CLD impl
    Some(&CMP_INDEXED_ABSOLUTE_Y), // 0xD9:CMP abs,Y
    None, // 0xDA:---
    None, // 0xDB:---
    None, // 0xDC:---
    Some(&CMP_INDEXED_ABSOLUTE_X), // 0xDD:CMP abs,X
    Some(&DEC_INDEXED_ABSOLUTE_X), // 0xDE:DEC abs,X
    None, // 0xDF:---
    Some(&CPX_IMMEDIATE), // 0xE0:CPX #
    Some(&SBC_INDEXED_INDIRECT_X), // 0xE1:SBC X,ind
    None, // 0xE2:---
    None, // 0xE3:---
    Some(&CPX_ZEROPAGE), // 0xE4:CPX zpg
    Some(&SBC_ZEROPAGE), // 0xE5:SBC zpg
    Some(&INC_ZEROPAGE), // 0xE6:INC zpg
    None, // 0xE7:---
    Some(&INX), // 0xE8:INX impl
    Some(&SBC_IMMEDIATE), // 0xE9:SBC #
    Some(&NOP), // 0xEA:NOP impl
    None, // 0xEB:---
    Some(&CPX_ABSOLUTE), // 0xEC:CPX abs
    Some(&SBC_ABSOLUTE), // 0xED:SBC abs
    Some(&INC_ABSOLUTE), // 0xEE:INC abs
    None, // 0xEF:---
    Some(&BEQ), // 0xF0:BEQ rel
    Some(&SBC_INDIRECT_INDEXED_Y), // 0xF1:SBC ind,Y
    None, // 0xF2:---
    None, // 0xF3:---
    None, // 0xF4:---
    Some(&SBC_INDEXED_ZEROPAGE_X), // 0xF5:SBC zpg,X
    Some(&INC_INDEXED_ZEROPAGE_X), // 0xF6:INC zpg,X
    None, // 0xF7:---
    Some(&SED), // 0xF8:SED impl
    Some(&SBC_INDEXED_ABSOLUTE_Y), // 0xF9:SBC abs,Y
    None, // 0xFA:---
    None, // 0xFB:---
    None, // 0xFC:---
    Some(&SBC_INDEXED_ABSOLUTE_X), // 0xFD:SBC abs,X
    Some(&INC_INDEXED_ABSOLUTE_X), // 0xFE:INC abs,X
    None, // 0xFF:---
];

// *********** DUMMY ***********
pub const DUMMY_INSTRUCTION: Instruction = new_instruction!(&IS_TEMP_DUMMY, &IS_DUMMY);

// *********** ORA ***********
const ORA_INDEXED_INDIRECT_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_INDIRECT_X, &IS_ORA);
const ORA_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE, &IS_ORA);
const ORA_IMMEDIATE: Instruction = new_instruction!(&IS_TEMP_IMMEDIATE, &IS_ORA);
const ORA_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE, &IS_ORA);
const ORA_INDIRECT_INDEXED_Y: Instruction = new_instruction!(&IS_TEMP_INDIRECT_INDEXED_Y, &IS_ORA);
const ORA_INDEXED_ZEROPAGE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ZEROPAGE_X, &IS_ORA);
const ORA_INDEXED_ABSOLUTE_Y: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_Y, &IS_ORA);
const ORA_INDEXED_ABSOLUTE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_X, &IS_ORA);

// *********** AND ***********
const AND_INDEXED_INDIRECT_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_INDIRECT_X, &IS_AND);
const AND_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE, &IS_AND);
const AND_IMMEDIATE: Instruction = new_instruction!(&IS_TEMP_IMMEDIATE, &IS_AND);
const AND_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE, &IS_AND);
const AND_INDIRECT_INDEXED_Y: Instruction = new_instruction!(&IS_TEMP_INDIRECT_INDEXED_Y, &IS_AND);
const AND_INDEXED_ZEROPAGE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ZEROPAGE_X, &IS_AND);
const AND_INDEXED_ABSOLUTE_Y: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_Y, &IS_AND);
const AND_INDEXED_ABSOLUTE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_X, &IS_AND);

// *********** EOR ***********
const EOR_INDEXED_INDIRECT_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_INDIRECT_X, &IS_EOR);
const EOR_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE, &IS_EOR);
const EOR_IMMEDIATE: Instruction = new_instruction!(&IS_TEMP_IMMEDIATE, &IS_EOR);
const EOR_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE, &IS_EOR);
const EOR_INDIRECT_INDEXED_Y: Instruction = new_instruction!(&IS_TEMP_INDIRECT_INDEXED_Y, &IS_EOR);
const EOR_INDEXED_ZEROPAGE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ZEROPAGE_X, &IS_EOR);
const EOR_INDEXED_ABSOLUTE_Y: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_Y, &IS_EOR);
const EOR_INDEXED_ABSOLUTE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_X, &IS_EOR);

// *********** ADC ***********
const ADC_INDEXED_INDIRECT_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_INDIRECT_X, &IS_ADC);
const ADC_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE, &IS_ADC);
const ADC_IMMEDIATE: Instruction = new_instruction!(&IS_TEMP_IMMEDIATE, &IS_ADC);
const ADC_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE, &IS_ADC);
const ADC_INDIRECT_INDEXED_Y: Instruction = new_instruction!(&IS_TEMP_INDIRECT_INDEXED_Y, &IS_ADC);
const ADC_INDEXED_ZEROPAGE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ZEROPAGE_X, &IS_ADC);
const ADC_INDEXED_ABSOLUTE_Y: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_Y, &IS_ADC);
const ADC_INDEXED_ABSOLUTE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_X, &IS_ADC);

// *********** STA ***********
const STA_INDEXED_INDIRECT_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_INDIRECT_X, &IS_STA);
const STA_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE, &IS_STA);
const STA_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE, &IS_STA);
const STA_INDIRECT_INDEXED_Y: Instruction = new_instruction!(&IS_TEMP_INDIRECT_INDEXED_Y, &IS_STA);
const STA_INDEXED_ZEROPAGE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ZEROPAGE_X, &IS_STA);
const STA_INDEXED_ABSOLUTE_Y: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_Y, &IS_STA);
const STA_INDEXED_ABSOLUTE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_X, &IS_STA);

// *********** LDA ***********
const LDA_INDEXED_INDIRECT_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_INDIRECT_X, &IS_LDA);
const LDA_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE, &IS_LDA);
const LDA_IMMEDIATE: Instruction = new_instruction!(&IS_TEMP_IMMEDIATE, &IS_LDA);
const LDA_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE, &IS_LDA);
const LDA_INDIRECT_INDEXED_Y: Instruction = new_instruction!(&IS_TEMP_INDIRECT_INDEXED_Y, &IS_LDA);
const LDA_INDEXED_ZEROPAGE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ZEROPAGE_X, &IS_LDA);
const LDA_INDEXED_ABSOLUTE_Y: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_Y, &IS_LDA);
const LDA_INDEXED_ABSOLUTE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_X, &IS_LDA);

// *********** CMP ***********
const CMP_INDEXED_INDIRECT_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_INDIRECT_X, &IS_CMP);
const CMP_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE, &IS_CMP);
const CMP_IMMEDIATE: Instruction = new_instruction!(&IS_TEMP_IMMEDIATE, &IS_CMP);
const CMP_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE, &IS_CMP);
const CMP_INDIRECT_INDEXED_Y: Instruction = new_instruction!(&IS_TEMP_INDIRECT_INDEXED_Y, &IS_CMP);
const CMP_INDEXED_ZEROPAGE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ZEROPAGE_X, &IS_CMP);
const CMP_INDEXED_ABSOLUTE_Y: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_Y, &IS_CMP);
const CMP_INDEXED_ABSOLUTE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_X, &IS_CMP);

// *********** SBC ***********
const SBC_INDEXED_INDIRECT_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_INDIRECT_X, &IS_SBC);
const SBC_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE, &IS_SBC);
const SBC_IMMEDIATE: Instruction = new_instruction!(&IS_TEMP_IMMEDIATE, &IS_SBC);
const SBC_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE, &IS_SBC);
const SBC_INDIRECT_INDEXED_Y: Instruction = new_instruction!(&IS_TEMP_INDIRECT_INDEXED_Y, &IS_SBC);
const SBC_INDEXED_ZEROPAGE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ZEROPAGE_X, &IS_SBC);
const SBC_INDEXED_ABSOLUTE_Y: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_Y, &IS_SBC);
const SBC_INDEXED_ABSOLUTE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_X, &IS_SBC);

// *********** STX ***********
const STX_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE, &IS_STX);
const STX_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE, &IS_STX);
const STX_INDEXED_ZEROPAGE_Y: Instruction = new_instruction!(&IS_TEMP_INDEXED_ZEROPAGE_Y, &IS_STX);

// *********** LDX ***********
const LDX_IMMEDIATE: Instruction = new_instruction!(&IS_TEMP_IMMEDIATE, &IS_LDX);
const LDX_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE, &IS_LDX);
const LDX_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE, &IS_LDX);
const LDX_INDEXED_ZEROPAGE_Y: Instruction = new_instruction!(&IS_TEMP_INDEXED_ZEROPAGE_Y, &IS_LDX);
const LDX_INDEXED_ABSOLUTE_Y: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_Y, &IS_LDX);

// *********** ASL ***********
const ASL_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE_RMW, &IS_ASL);
const ASL_ACCUMULATOR: Instruction = new_instruction!(&IS_TEMP_ACCUMULATOR_RMW, &IS_ASL);
const ASL_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE_RMW, &IS_ASL);
const ASL_INDEXED_ZEROPAGE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ZEROPAGE_X_RMW, &IS_ASL);
const ASL_INDEXED_ABSOLUTE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_X_RMW, &IS_ASL);

// *********** ROL ***********
const ROL_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE_RMW, &IS_ROL);
const ROL_ACCUMULATOR: Instruction = new_instruction!(&IS_TEMP_ACCUMULATOR_RMW, &IS_ROL);
const ROL_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE_RMW, &IS_ROL);
const ROL_INDEXED_ZEROPAGE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ZEROPAGE_X_RMW, &IS_ROL);
const ROL_INDEXED_ABSOLUTE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_X_RMW,&IS_ROL);

// *********** LSR ***********
const LSR_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE_RMW, &IS_LSR);
const LSR_ACCUMULATOR: Instruction = new_instruction!(&IS_TEMP_ACCUMULATOR_RMW, &IS_LSR);
const LSR_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE_RMW, &IS_LSR);
const LSR_INDEXED_ZEROPAGE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ZEROPAGE_X_RMW, &IS_LSR);
const LSR_INDEXED_ABSOLUTE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_X_RMW, &IS_LSR);

// *********** ROR ***********
const ROR_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE_RMW, &IS_ROR);
const ROR_ACCUMULATOR: Instruction = new_instruction!(&IS_TEMP_ACCUMULATOR_RMW, &IS_ROR);
const ROR_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE_RMW, &IS_ROR);
const ROR_INDEXED_ZEROPAGE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ZEROPAGE_X_RMW, &IS_ROR);
const ROR_INDEXED_ABSOLUTE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_X_RMW, &IS_ROR);

// *********** DEC ***********
const DEC_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE_RMW, &IS_DEC);
const DEC_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE_RMW, &IS_DEC);
const DEC_INDEXED_ZEROPAGE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ZEROPAGE_X_RMW, &IS_DEC);
const DEC_INDEXED_ABSOLUTE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_X_RMW, &IS_DEC);

// *********** INC ***********
const INC_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE_RMW, &IS_INC);
const INC_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE_RMW, &IS_INC);
const INC_INDEXED_ZEROPAGE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ZEROPAGE_X_RMW, &IS_INC);
const INC_INDEXED_ABSOLUTE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_X_RMW, &IS_INC);

// *********** BIT ***********
const BIT_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE, &IS_BIT);
const BIT_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE, &IS_BIT);

// *********** JMP ***********
const JMP_INDIRECT: Instruction = new_instruction!(&IS_TEMP_INDIRECT_JMP, &IS_JMP);
const JMP_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE_JMP, &IS_JMP);

// *********** STY ***********
const STY_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE, &IS_STY);
const STY_INDEXED_ZEROPAGE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ZEROPAGE_X, &IS_STY);
const STY_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE, &IS_STY);

// *********** LDY ***********
const LDY_IMMEDIATE: Instruction = new_instruction!(&IS_TEMP_IMMEDIATE, &IS_LDY);
const LDY_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE, &IS_LDY);
const LDY_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE, &IS_LDY);
const LDY_INDEXED_ZEROPAGE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ZEROPAGE_X, &IS_LDY);
const LDY_INDEXED_ABSOLUTE_X: Instruction = new_instruction!(&IS_TEMP_INDEXED_ABSOLUTE_X, &IS_LDY);

// *********** CPY ***********
const CPY_IMMEDIATE: Instruction = new_instruction!(&IS_TEMP_IMMEDIATE, &IS_CPY);
const CPY_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE, &IS_CPY);
const CPY_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE, &IS_CPY);

// *********** CPX ***********
const CPX_IMMEDIATE: Instruction = new_instruction!(&IS_TEMP_IMMEDIATE, &IS_CPX);
const CPX_ZEROPAGE: Instruction = new_instruction!(&IS_TEMP_ZEROPAGE, &IS_CPX);
const CPX_ABSOLUTE: Instruction = new_instruction!(&IS_TEMP_ABSOLUTE, &IS_CPX);

// *********** BPL ***********
const BPL: Instruction = new_instruction!(&IS_TEMP_RELATIVE, &IS_BPL);
// *********** BMI ***********
const BMI: Instruction = new_instruction!(&IS_TEMP_RELATIVE, &IS_BMI);
// *********** BVC ***********
const BVC: Instruction = new_instruction!(&IS_TEMP_RELATIVE, &IS_BVC);
// *********** BVS ***********
const BVS: Instruction = new_instruction!(&IS_TEMP_RELATIVE, &IS_BVS);
// *********** BCC ***********
const BCC: Instruction = new_instruction!(&IS_TEMP_RELATIVE, &IS_BCC);
// *********** BCS ***********
const BCS: Instruction = new_instruction!(&IS_TEMP_RELATIVE, &IS_BCS);
// *********** BNE ***********
const BNE: Instruction = new_instruction!(&IS_TEMP_RELATIVE, &IS_BNE);
// *********** BEQ ***********
const BEQ: Instruction = new_instruction!(&IS_TEMP_RELATIVE, &IS_BEQ);
// *********** JSR ***********
const JSR: Instruction = new_instruction!(&IS_TEMP_JSR, &IS_JSR);
// *********** RTI ***********
const RTI: Instruction = new_instruction!(&IS_TEMP_RTI, &IS_RTI);
// *********** RTS ***********
const RTS: Instruction = new_instruction!(&IS_TEMP_RTS, &IS_RTS);
// *********** PHP ***********
const PHP: Instruction = new_instruction!(&IS_TEMP_PUSH_STACK, &IS_PHP);
// *********** PLP ***********
const PLP: Instruction = new_instruction!(&IS_TEMP_PULL_STACK, &IS_PLP);
// *********** PHA ***********
const PHA: Instruction = new_instruction!(&IS_TEMP_PUSH_STACK, &IS_PHA);
// *********** PLA ***********
const PLA: Instruction = new_instruction!(&IS_TEMP_PULL_STACK, &IS_PLA);
// *********** DEY ***********
const DEY: Instruction = new_instruction!(&IS_TEMP_IMPLIED, &IS_DEY);
// *********** TAY ***********
const TAY: Instruction = new_instruction!(&IS_TEMP_IMPLIED, &IS_TAY);
// *********** INY ***********
const INY: Instruction = new_instruction!(&IS_TEMP_IMPLIED, &IS_INY);
// *********** INX ***********
const INX: Instruction = new_instruction!(&IS_TEMP_IMPLIED, &IS_INX);
// *********** CLC ***********
const CLC: Instruction = new_instruction!(&IS_TEMP_IMPLIED, &IS_CLC);
// *********** SEC ***********
const SEC: Instruction = new_instruction!(&IS_TEMP_IMPLIED, &IS_SEC);
// *********** CLI ***********
const CLI: Instruction = new_instruction!(&IS_TEMP_IMPLIED, &IS_CLI);
// *********** SEI ***********
const SEI: Instruction = new_instruction!(&IS_TEMP_IMPLIED, &IS_SEI);
// *********** TYA ***********
const TYA: Instruction = new_instruction!(&IS_TEMP_IMPLIED, &IS_TYA);
// *********** CLV ***********
const CLV: Instruction = new_instruction!(&IS_TEMP_IMPLIED, &IS_CLV);
// *********** CLD ***********
const CLD: Instruction = new_instruction!(&IS_TEMP_IMPLIED, &IS_CLD);
// *********** SED ***********
const SED: Instruction = new_instruction!(&IS_TEMP_IMPLIED, &IS_SED);
// *********** TXA ***********
const TXA: Instruction = new_instruction!(&IS_TEMP_IMPLIED, &IS_TXA);
// *********** TXS ***********
const TXS: Instruction = new_instruction!(&IS_TEMP_IMPLIED, &IS_TXS);
// *********** TAX ***********
const TAX: Instruction = new_instruction!(&IS_TEMP_IMPLIED, &IS_TAX);
// *********** TSX ***********
const TSX: Instruction = new_instruction!(&IS_TEMP_IMPLIED, &IS_TSX);
// *********** DEX ***********
const DEX: Instruction = new_instruction!(&IS_TEMP_IMPLIED, &IS_DEX);
// *********** NOP ***********
const NOP: Instruction = new_instruction!(&IS_TEMP_IMPLIED, &IS_NOP);
