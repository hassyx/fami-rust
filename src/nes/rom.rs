//! NES rom data container and utilities.

use std::fs::File;
use std::io::Read;
// use std::io::Result;
// use std::io::Error;
use std::error::Error;

use crate::nes::util;

pub const PRG_ROM_UNIT_SIZE: usize = 0x4000;
pub const CHR_ROM_UNIT_SIZE: usize = 0x2000;

/// NESのROM(バイナリ)情報を保存する構造体。
/// バイナリの構成については以下を参照。
/// https://wiki.nesdev.org/w/index.php/INES
/// https://wiki.nesdev.org/w/index.php?title=NES_2.0
pub struct NesRom {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    trainer: Option<Vec<u8>>,
    mirroring_type: MirroringType,
    battery_backed: bool,
    console_type: ConsoleType,
    mapper_no: u16,
    prg_ram_size: u32,   // TODO:最大サイズは？
    eeprom_size: u32,
    tv_format: TvFormat,
    chr_ram_size: u32,
    chr_nvram_size: u32,
    cpu_timing: CPUTiming,
    vssystem_type: u8,
    vshardware_type: u8,
}

impl NesRom {
    pub fn prg_rom(&self) -> &[u8] {
        &self.prg_rom
    }

    pub fn chr_rom(&self) -> &[u8] {
        &self.chr_rom
    }

    pub fn mirroring_type(&self) -> MirroringType {
        self.mirroring_type
    }
}

/// ネームテーブルのミラーリングタイプ
#[derive(Copy, Clone)]
pub enum MirroringType {
    None,
    Horizontal,
    Vertical,
}

pub enum ConsoleType {
    Nes,
    VsSystem,
    Playchoice10,
    Extended,
}

pub enum TvFormat {
    NTSC,
    PAL,
}

pub enum CPUTiming {
    NTSC,
    PAL,
    MultiRegion,
    Dendy,
}

pub fn load_from_file(path: &str) -> Result<Box<NesRom>, Box<dyn Error>> {
    //let mut file = File::open(path)?;
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(err) => {
            let msg = format!("[{}] {}", path, err);
            return Err(util::Error::new(msg));
        }
    };
    let mut buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut buf)?;
    match parse(&buf) {
        Ok(rom) => Ok(rom),
        Err(msg) => Err(util::Error::new(format!("[{}] {}", path, msg))),
    }
}

fn parse(rom_bin: &Vec<u8>) -> Result<Box<NesRom>, &str>
 {
    // NESファイルを読み込んで解析する
    // 対応するファイルのフォーマットは NES2.0 とする(つまりiNESもサポート)。
    // https://wiki.nesdev.org/w/index.php?title=NES_2.0

    const HEADER_LEN: usize = 16;

    // バイト 0-4
    let header = 
        if rom_bin.len() >= HEADER_LEN {
            &rom_bin[..HEADER_LEN]
        } else {
            return Err("Header size is too short.");
        };

    if  header[0] != 0x4E ||
        header[1] != 0x45 ||
        header[2] != 0x53 ||
        header[3] != 0x1A
    {
        return Err("Invalid format.");
    }

    let prg_lower = header[4];
    let chr_lower = header[5];
    // バイト 6
    let (mirroring_type,
        battery_backed,
        has_trainer,
        mapper_lower) = parse_flag6(header[6]);
    // バイト 7
    let (console_type,
        is_nes_2_0,
        mapper_middle) = parse_flag7(header[7]);
    
    let prg_ram_size: u32;
    let eeprom_size: u32;
    let mapper_upper: u8;
    let submapper: u8;
    
    if is_nes_2_0 {
        // バイト 8, 10
        let (prg, eep) = parse_flag10_v2(header[10]);
        prg_ram_size = prg;
        eeprom_size = eep;
        let (upper, sub) = parse_flag8_v2(header[8]);
        mapper_upper = upper;
        submapper = sub;
    } else {
        // バイト 8
        prg_ram_size = parse_flag8(header[8]);
        eeprom_size = 0;
        mapper_upper = 0;
        submapper = 0;
    }
    
    let mapper_no: u16 = {
        let upper = mapper_upper as u16;
        let middle = mapper_middle as u16;
        let lower = mapper_lower as u16;
        (upper << 8) | (middle << 4) | lower
    };
    
    let tv_format: TvFormat;
    let prg_rom_size: usize;
    let chr_rom_size: usize;
    // バイト 9
    if is_nes_2_0 {
        let (prg_upper, chr_upper) = parse_flag9_v2(header[9]);
        prg_rom_size = calculate_rom_size(prg_upper, prg_lower);
        chr_rom_size = calculate_rom_size(chr_upper, chr_lower);
        tv_format = TvFormat::NTSC;
    } else  {
        tv_format = parse_flag9(header[9]);
        prg_rom_size = (prg_lower as usize) * PRG_ROM_UNIT_SIZE;
        chr_rom_size = (chr_lower as usize) * CHR_ROM_UNIT_SIZE;
    }

    let chr_ram_size: u32;
    let chr_nvram_size: u32;
    // バイト 11
    if is_nes_2_0 {
        let (chr, chr_nv) = parse_flag11_v2(header[11]);
        chr_ram_size = chr;
        chr_nvram_size = chr_nv;
    } else{
        chr_ram_size = 0;
        chr_nvram_size = 0;
    }

    // バイト 12
    let cpu_timing = 
        if is_nes_2_0 {
            parse_flag12_v2(header[12])
        } else {
            // デフォルトはNTSCにしておく
            CPUTiming::NTSC
        };

    // バイト 13
    let (vssystem_type, vshardware_type) =
        if is_nes_2_0 {
            parse_flag13_v2(header[13])
        } else {
             (0, 0)
        };
    
    // 当面は無視
    // バイト 14
    let misc_rom_count = 
        if is_nes_2_0 {
            parse_flag14_v2(header[13])
        } else {
            0
        };

    // 当面は無視
    // バイト 15
    let expansion_device =
        if is_nes_2_0 {
            parse_flag15_v2(header[13])
        } else {
            0
        };

    let mut index = HEADER_LEN;

    // トレーナー領域
    let trainer: Option<Vec<u8>> = 
        if has_trainer {
            const TRAINLER_LEN: usize = 512;
            let start = index;
            index += TRAINLER_LEN;
            let mut dst = Vec::<u8>::with_capacity(TRAINLER_LEN);
            dst.resize(TRAINLER_LEN, 0);
            dst.copy_from_slice(&rom_bin[start..TRAINLER_LEN+start]);
            Some(dst)
        } else {
            None
        };
    
    // PRG-ROM領域
    let prg_rom: Vec<u8> = {
        let start = index;
        index += prg_rom_size;
        let mut dst = Vec::<u8>::with_capacity(prg_rom_size);
        dst.resize(prg_rom_size, 0);
        dst.copy_from_slice(&rom_bin[start..prg_rom_size+start]);
        dst
    };
        
    // CHR-ROM領域
    let chr_rom: Vec<u8> = {
        let start = index;
        index += chr_rom_size;
        let mut dst = Vec::<u8>::with_capacity(chr_rom_size);
        dst.resize(chr_rom_size, 0);
        dst.copy_from_slice(&rom_bin[start..chr_rom_size+start]);
        dst
    };

    // これ以降はPlayChoice用のデータ等が存在する場合がある。
    // ひとまず無視。

    Ok(Box::new(NesRom {
        prg_rom,
        chr_rom,
        trainer,
        mirroring_type,
        battery_backed,
        console_type,
        mapper_no,
        prg_ram_size,
        eeprom_size,
        tv_format,
        chr_ram_size,
        chr_nvram_size,
        cpu_timing,
        vssystem_type,
        vshardware_type,
    }))
 }

fn parse_flag6(flags: u8) -> (MirroringType, bool, bool, u8) {
    // Flags 6
    // D~7654 3210
    //   ---------
    //   NNNN FTBM
    //   |||| |||+-- Hard-wired nametable mirroring type
    //   |||| |||     0: Horizontal or mapper-controlled
    //   |||| |||     1: Vertical
    //   |||| ||+--- "Battery" and other non-volatile memory
    //   |||| ||      0: Not present
    //   |||| ||      1: Present
    //   |||| |+--- 512-byte Trainer
    //   |||| |      0: Not present
    //   |||| |      1: Present between Header and PRG-ROM data
    //   |||| +---- Hard-wired four-screen mode
    //   ||||        0: No
    //   ||||        1: Yes
    //   ++++------ Mapper Number D0..D3

    let mirroring_type = 
        if (flags & 0b0000_1000) != 0 {
            MirroringType::None
        } else if (flags & 0b0000_0001) != 0 {
            MirroringType::Vertical
        } else {
            MirroringType::Horizontal
        };
    
    let battery_backed = (flags & 0b0000_0010) != 0;
    let has_trainer = (flags & 0b0000_0100) != 0;
    let mapper_lower = (flags & 0b1111_0000) >> 4;

    (mirroring_type, battery_backed, has_trainer, mapper_lower)
}

fn parse_flag7(flags: u8) -> (ConsoleType, bool, u8) {
    // Flags 7
    // D~7654 3210
    //   ---------
    //   NNNN 10TT
    //   |||| ||++- Console type
    //   |||| ||     0: Nintendo Entertainment System/Family Computer
    //   |||| ||     1: Nintendo Vs. System
    //   |||| ||     2: Nintendo Playchoice 10
    //   |||| ||     3: Extended Console Type
    //   |||| ++--- NES 2.0 identifier
    //   ++++------ Mapper Number D4..D7

    let console_type = match flags & 0b0000_0011 {
        0b00 => ConsoleType::Nes,
        0b01 => ConsoleType::VsSystem,
        0b10 => ConsoleType::Playchoice10,
        0b11 => ConsoleType::Extended,
        _ => ConsoleType::Nes,
    };

    let is_nes_2_0 = ((flags & 0b0000_1100) >> 2) == 2;
    let mapper_upper = (flags & 0b1111_0000) >> 4;

    (console_type, is_nes_2_0, mapper_upper)
}

fn parse_flag8(flags: u8) -> u32 {
    // 76543210
    // ||||||||
    // ++++++++- PRG RAM size (Value 0 infers 8 KB for compatibility)
    
    if flags == 0 {
        return 0x2000
    } else {
        return 0x2000 * (flags as u32)
    }
}

fn parse_flag8_v2(flags: u8) -> (u8, u8) {
    // Mapper MSB/Submapper
    // D~7654 3210
    //   ---------
    //   SSSS NNNN
    //   |||| ++++- Mapper number D8..D11
    //   ++++------ Submapper number

    let mapper_upper = flags & 0b0000_1111;
    let submapper = flags & 0b0000_1111;

    return (mapper_upper, submapper)
}

fn parse_flag9(flags: u8) -> TvFormat {
    // 76543210
    // ||||||||
    // |||||||+- TV system (0: NTSC; 1: PAL)
    // +++++++-- Reserved, set to zero

    if (flags & 0x0000_0001) == 0 {
        return TvFormat::NTSC
    } else {
        return TvFormat::PAL
    }
}

fn parse_flag9_v2(flags: u8) -> (u8, u8) {
    // PRG-ROM/CHR-ROM size MSB
    // D~7654 3210
    // ---------
    // CCCC PPPP
    // |||| ++++- PRG-ROM size MSB
    // ++++------ CHR-ROM size MSB

    let prg_rom_size = flags & 0b0000_1111;
    let chr_rom_size = flags & 0b1111_0000;
    return (prg_rom_size, chr_rom_size)
}

/*
fn parse_flag10(flags: u8) -> TvFormat {
    // 76543210
    // ||  ||
    // ||  ++- TV system (0: NTSC; 2: PAL; 1/3: dual compatible)
    // |+----- PRG RAM ($6000-$7FFF) (0: present; 1: not present)
    // +------ 0: Board has no bus conflicts; 1: Board has bus conflicts

    if (flags & 0x0000_0001) == 0 {
        return TvFormat::NTSC
    } else {
        return TvFormat::PAL
    }
}
*/

fn parse_flag10_v2(flags: u8) -> (u32, u32) {
    // PRG-RAM/EEPROM size
    // D~7654 3210
    //   ---------
    //   pppp PPPP
    //   |||| ++++- PRG-RAM (volatile) shift count
    //   ++++------ PRG-NVRAM/EEPROM (non-volatile) shift count
    // If the shift count is zero, there is no PRG-(NV)RAM.
    // If the shift count is non-zero, the actual size is
    // "64 << shift count" bytes, i.e. 8192 bytes for a shift count of 7.
    
    let flags = flags as u32;
    let prg_shift: u32 = (flags >> 4) & 0b0000_1111;
    let eep_shift: u32 = flags & 0b0000_1111;

    return (64 << prg_shift, 64 << eep_shift);
}

fn parse_flag11_v2(flags: u8) -> (u32, u32) {
    // CHR-RAM size
    // D~7654 3210
    //   ---------
    //   cccc CCCC
    //   |||| ++++- CHR-RAM size (volatile) shift count
    //   ++++------ CHR-NVRAM size (non-volatile) shift count
    // If the shift count is zero, there is no CHR-(NV)RAM.
    // If the shift count is non-zero, the actual size is
    // "64 << shift count" bytes, i.e. 8192 bytes for a shift count of 7.

    let flags = flags as u32;
    let chr_shift: u32 = (flags >> 4) & 0b0000_1111;
    let chr_nv_shift: u32 = flags & 0b0000_1111;

    return (64 << chr_shift, 64 << chr_nv_shift);
}

fn parse_flag12_v2(flags: u8) -> CPUTiming {
    //CPU/PPU Timing
    //D~7654 3210
    //  ---------
    //  .... ..VV
    //         ++- CPU/PPU timing mode
    //              0: RP2C02 ("NTSC NES")
    //              1: RP2C07 ("Licensed PAL NES")
    //              2: Multiple-region
    //              3: UMC 6527P ("Dendy")

    match flags | 0b0000_0011 {
        0b00 => CPUTiming::NTSC,
        0b01 => CPUTiming::PAL,
        0b10 => CPUTiming::MultiRegion,
        0b11 => CPUTiming::Dendy,
        _ => CPUTiming::NTSC,
    }
}

fn parse_flag13_v2(flags: u8) -> (u8, u8) {
    //When Byte 7 AND 3 =1: Vs. System Type
    //D~7654 3210
    //  ---------
    //  MMMM PPPP
    //  |||| ++++- Vs. PPU Type
    //  ++++------ Vs. Hardware Type

    //When Byte 7 AND 3 =3: Extended Console Type
    //D~7654 3210
    //  ---------
    //  .... CCCC
    //       ++++- Extended Console Type

    // vssystem = 1 の場合にのみ必要な値だが、特に気にせず返す
    let vssystem_type = flags & 0b0000_1111;
    let vshardware_type = (flags >> 4) & 0b0000_1111;

    return (vssystem_type, vshardware_type)
}

// 当面は無視
fn parse_flag14_v2(flags: u8) -> u8 {
    //Miscellaneous ROMs
    //D~7654 3210
    //  ---------
    //  .... ..RR
    //         ++- Number of miscellaneous ROMs present

    flags & 0b0000_0011
}

// 当面は無視
fn parse_flag15_v2(flags: u8) -> u8 {
    //Default Expansion Device
    //D~7654 3210
    //  ---------
    //  ..DD DDDD
    //    ++-++++- Default Expansion Device

    flags & 0b0011_1111
}

fn calculate_rom_size(upper: u8, lower: u8) -> usize {
    let rom_size: usize =
        if upper == 0b0000_1111 {
            // If the MSB nibble is $F, an exponent-multiplier notation is used:
            let mm = (lower & 0b0000_0011) as u32;
            let exponent = ((lower >> 2) & 0b0011_1111) as u32;
            // 2^E *(MM*2+1) bytes.
            (2u32.pow(exponent) * (mm * 2 + 1)) as usize
        } else {
            (upper as usize) << 8 | (lower as usize)
        };
    
    rom_size
}