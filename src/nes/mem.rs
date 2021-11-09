//! CPU側の Memory Controller。
//! ミラー領域への値の反映など、メモリへの読み書きを仲介する。

use std::rc::Rc;
use crate::nes::cpu;
use crate::nes::ppu;

/// NESに搭載されているRAM容量(bytes)
pub const REAL_RAM_SIZE: usize = 2048;
/// メモリ空間の広さ(bytes)
pub const RAM_SPACE: usize = 16384;

pub struct MemCon {
    cpu_regs: Rc<cpu::Registers>,
    ppu_regs: Rc<ppu::Registers>,
    ram: Box<[u8]>,
}

impl MemCon {
    pub fn new(cpu_regs: Rc<cpu::Registers>, ppu_regs: Rc<ppu::Registers>) -> MemCon {
        MemCon {
            cpu_regs,
            ppu_regs,
            ram: Box::new([0; RAM_SPACE]),
        }
    }
}