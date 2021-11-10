//! CPU側の Memory Controller。
//! ミラー領域への値の反映など、メモリへの読み書きを仲介する。

use crate::nes::cpu;
use crate::nes::ppu;

/// NESに搭載されている物理RAM容量(bytes)
pub const REAL_RAM_SIZE: usize = 0x0800;
/// メモリ空間の広さ(bytes)
pub const RAM_SPACE: usize = 0xFFFF;

pub struct MemCon<'a> {
    ram: Box<[u8]>,
    cpu_regs: &'a cpu::Registers,
    ppu_regs: &'a ppu::Registers,
}

/*
impl Default for MemCon {
    fn default() -> Self {
        Self {
            ram: Box::new([0; RAM_SPACE]),
        }
    }
}
*/

impl<'a> MemCon<'a> {
    pub fn new(cpu_regs: &'a cpu::Registers, ppu_regs: &'a ppu::Registers) -> Self {
        MemCon {
            cpu_regs,
            ppu_regs,
            ram: Box::new([0; RAM_SPACE]),
        }
    }

    /// ミラーリング等を考慮せず、メモリに直にデータを書き込む。
    /// 主に初期化処理に利用する。
    pub fn raw_write(&mut self, addr: usize, data: &[u8]) {
        println!("addr={}, data.len()={}", addr, data.len());

        self.ram[addr..addr+data.len()].copy_from_slice(data);
    }

    
}