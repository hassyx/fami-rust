//! NES PPU.

use std::rc::Rc;
use crate::nes::vram;

/// スプライト用メモリ容量(bytes)
pub const SPR_RAM_SIZE: usize = 256;

pub struct PPU {
    regs: Rc<Registers>,
    
    /// スプライト用のメモリ(256バイト)
    /// VRAMと違い、特別な対応が必要ないのでベタな配列として扱う。
    spr_ram: Box<[u8]>,

    /// VRAMへのアクセスを司るコントローラ
    vram: Box<vram::MemCon>
}

#[derive(Default)]
pub struct Registers {
    /// PPUCTRL($2000): PPU制御用のフラグレジスタ
    ctrl: u8,
    /// PPUMASK($2001): マスク処理用のレジスタ
    mask: u8,
    /// PPUSTATUS ($2002): ステータスレジスタ
    status: u8,
    /// OAMADDR ($2003): OAM(SPR-RAM)への書き込み先アドレス設定用のレジスタ
    oam_addr: u8,
    /// OAMDATA ($2004): OAM(SPR-RAM)への読み書きレジスタ
    oam_data: u8,
    /// PPUSCROLL ($2005): スクロール位置変更用レジスタ
    scroll: u8,
    /// PPUADDR ($2006): VRAMへの書き込み位置の指定用レジスタ
    addr: u8,
    /// PPUDATA ($2007): VRAMへの書き込みと読み込み用レジスタ
    data: u8,
    /// OAMDMA ($4014): OAM(SPR-RAM)へのDMA転送に使用する、
    /// source(CPU側のRAM)のアドレスを指定するレジスタ
    oam_dma: u8,
}

impl Default for PPU {
    fn default() -> Self {
        let regs = Rc::new(Registers::default());
        Self {
            regs: Rc::clone(&regs),
            spr_ram: Box::new([0; SPR_RAM_SIZE]),
            vram: Box::new(vram::MemCon::new(Rc::clone(&regs))),
        }
    }
}

impl PPU {
    pub fn exec(&self) -> u32 {
        // 
        100
    }

    /// 割り込み
    pub fn interrupt(&self) {
        // TODO: CPU側にVBlankを投げる必要あり。
    }

    pub fn registers(&self) -> Rc<Registers> {
        Rc::clone(&self.regs)
    }
}