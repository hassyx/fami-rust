//! PPUのVRAMを管理する Memory Controller。
//! ミラー領域への値の反映など、メモリへの読み書きを仲介する。

use std::rc::Rc;
use crate::nes::ppu;

/// PPUに搭載されているVRAM容量(bytes)
pub const REAL_VRAM_SIZE: usize = 16384;
/// メモリ空間の広さ(bytes)
pub const VRAM_SPACE: usize = 65536;

/// 64KBのメモリ空間を持ち、物理的には16KBの容量を持つVRAMのメモリコントローラー。
pub struct MemCon {
    ppu_regs: Rc<ppu::Registers>,
    ram: Box<[u8]>,
}

// TODO:!!!!!!!!!
// メソッドを介してではなく、普通の配列として振る舞うよう実装する。
// "&[..]" 記法でスライスも生成できるようにする。
// これはCPU側のメモリーもそうすべき。

/*
impl Default for MemCon {
    fn default() -> Self {
        Self {
            ram: Box::new([0; VRAM_SPACE]),
        }
    }
}
*/

impl MemCon {
    pub fn new(ppu_regs: Rc<ppu::Registers>) -> Self {
        Self {
            ppu_regs,
            ram: Box::new([0; VRAM_SPACE]),
        }
    }
}