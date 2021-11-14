//! CPUとPPUを繋ぐデータバス。

use std::cell::RefCell;
use std::rc::Rc;

use crate::nes::ppu::Ppu;

pub struct DataBus {
    ppu: Rc<RefCell<Ppu>>,
    latch: u8,
}

/*
pub enum PpuRegs {
    Ctrl,
    Mask,
    Status,
    OamAddr,
    OamData,
    Scroll,
    PpuAddr,
    PpuData,
    OamDma,
}
*/

impl DataBus {

    pub fn new(ppu: Rc<RefCell<Ppu>>) -> Self {
        Self {
            ppu,
            latch: 0,
        }
    }

    /// 注意：全てのレジスタについて、CPU側から書き込み、または読み込みを行うと、バス上にあるラッチも更新される。
    pub fn write(&mut self, addr: usize, data: u8, clock_count: u64) {
        let mut ppu = self.ppu.borrow_mut();
        // なんにせよデータバス上のラッチは必ず更新しておく
        ppu.regs.latch = data;
        // レジスタを更新
        // PPUのレジスタへの値の設定、かつミラー領域への反映
        match addr {
            0x2000 => if Ppu::is_ready(clock_count) { ppu.regs.ctrl = data },
            0x2001 => if Ppu::is_ready(clock_count) { ppu.regs.mask = data },
            0x2002 => (), // PPUSTATUSは読み込み専用
            0x2003 => ppu.regs.oam_addr = data,
            0x2004 => ppu.regs.oam_data = data,
            0x2005 => if Ppu::is_ready(clock_count) { ppu.regs.scroll = data },
            0x2006 => if Ppu::is_ready(clock_count) { ppu.regs.addr = data },
            0x2007 => ppu.regs.data = data,
            0x4014 => ppu.regs.oam_dma = data,
            _ => panic!("invalid address."),
        };
    }

    /// 注意：書き込み専用レジスタを読み込むと、レジスタではなく、現在のラッチの値を返す。
    pub fn read(&mut self, addr: usize, clock_count: u64) -> u8 {
        let ppu = self.ppu.borrow_mut();
        // 可能であればレジスタを読み込む。読み込み禁止の場合はラッチの値を返す。
        match addr {
            0x2000 => ppu.regs.latch,
            0x2001 => ppu.regs.latch,
            0x2002 => ppu.regs.status,
            0x2003 => ppu.regs.latch,
            0x2004 => ppu.regs.oam_data,
            0x2005 => ppu.regs.latch,
            0x2006 => ppu.regs.latch,
            0x2007 => ppu.regs.data,
            0x4014 => ppu.regs.latch,
            _ => panic!("invalid address."),
        }
    }
}
