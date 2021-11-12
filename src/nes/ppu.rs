//! NES PPU.

use crate::nes::rom;
use crate::nes::vram;

/// スプライト用メモリ容量(bytes)
pub const SPR_RAM_SIZE: usize = 256;

pub struct Ppu {
    pub regs: Registers,
    /// スプライト用のメモリ(256バイト)
    /// VRAMと違い、特別な対応が必要ないのでベタな配列として扱う。
    spr_ram: Box<[u8]>,
    /// VRAMへのアクセスを司るコントローラ
    vram: Box<vram::MemCon>,
    clock_count: u64,
}

impl Ppu {
    pub fn new(rom: &rom::NesRom) -> Ppu {
        let mut my = Ppu {
            regs: Default::default(),
            spr_ram: Box::new([0; SPR_RAM_SIZE]),
            vram: Box::new(vram::MemCon::new()),
            clock_count: 0,
        };
        
        {
            // CHR-ROM を VRAM に展開
            let chr_rom = rom.chr_rom();
            let len = rom::CHR_ROM_UNIT_SIZE;
            if chr_rom.len() >= len {
                my.vram.raw_write(0x0000, &chr_rom[0..len]);
            }
        }

        return my
    }

    pub fn power_on(&mut self) {
        // TODO: 起動後、約29658クロック以内は書き込みを無視する必要がある
        // https://wiki.nesdev.org/w/index.php/PPU_power_up_state
    }

    pub fn exec(&mut self) -> u32 {
        // 
        self.regs.status = 0;
        100
    }

    /// 割り込み
    pub fn interrupt(&self) {
        // TODO: CPU側にVBlankを投げる必要あり。
    }
}

#[derive(Default)]
pub struct Registers {
    /// PPUCTRL($2000): 書き込み専用。PPU制御用のフラグレジスタ。
    pub ctrl: u8,
    /// PPUMASK($2001): 書き込み専用。マスク処理用のレジスタ。
    pub mask: u8,
    /// PPUSTATUS ($2002): 読み込み専用。ステータスレジスタ。
    pub status: u8,
    /// OAMADDR ($2003): 書き込み専用。OAM(SPR-RAM)への書き込み先アドレス設定用のレジスタ。
    pub oam_addr: u8,
    /// OAMDATA ($2004): 読み書き可能。OAM(SPR-RAM)への読み書きレジスタ。
    pub oam_data: u8,
    /// PPUSCROLL ($2005): 書き込み専用。スクロール位置変更用レジスタ。
    pub scroll: u8,
    /// PPUADDR ($2006): 書き込み専用。VRAMへの書き込み位置の指定用レジスタ。
    pub addr: u8,
    /// PPUDATA ($2007): 読み書き可能。VRAMへの書き込みと読み込み用レジスタ。  
    pub data: u8,
    /// OAMDMA ($4014): 書き込み専用。OAM(SPR-RAM)へのDMA転送に使用する、
    /// source(CPU側のRAM)側のアドレスを指定するレジスタ。  
    pub oam_dma: u8,
    /// CPUとPPUのデータ転送に利用するバス。実体は8bitのラッチ。
    /// PPUのレジスタへ、CPU側から読み書きを行った場合に更新される。
    pub latch: u8,
}

/*
pub struct Reg {
    data: u8,
    flags: u8,
}

impl Reg {
    const IS_READABLE: u8 = 0b0000_0001;
    const IS_WRITABLE: u8 = 0b0000_0010;
    
    /// 外部デバイスからのレジスタへの書き込み
    fn ext_write() {

    }
}
*/
