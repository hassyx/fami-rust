//! CPUとPPUを繋ぐデータバス。

use std::cell::RefCell;
use std::rc::Rc;
use num_derive::FromPrimitive;    

use crate::nes::ppu::Ppu;

pub struct DataBus {
    ppu: Rc<RefCell<Ppu>>,
    latch: u8,
}

#[derive(FromPrimitive)]
pub enum PpuRegs {
    /// $2000
    Ctrl = 0,
    /// $2001
    Mask = 1,
    /// $2002
    Status = 2,
    /// $2003
    OamAddr = 3,
    /// $2004
    OamData = 4,
    /// $2005
    Scroll = 5,
    /// $2006
    PpuAddr = 6,
    /// $2007
    PpuData = 7,
}

impl DataBus {

    pub fn new(ppu: Rc<RefCell<Ppu>>) -> Self {
        Self {
            ppu,
            latch: 0,
        }
    }

    /// CPUからの、メモリを介したPPUへの書き込み要請
    pub fn write(&mut self, reg_type: PpuRegs, data: u8) {
        let mut ppu = self.ppu.borrow_mut();
        // バスを介した書き込みを行うと、ラッチも必ず更新される。
        self.latch = data;
        // PPUのレジスタへの値の設定、かつミラー領域への反映
        match reg_type {
            PpuRegs::Ctrl => if ppu.is_ready() { ppu.regs.ctrl = data },
            PpuRegs::Mask => if ppu.is_ready() { ppu.regs.mask = data },
            PpuRegs::Status => (), // PPUSTATUSは読み込み専用
            PpuRegs::OamAddr => ppu.regs.oam_addr = data,
            PpuRegs::OamData => ppu.regs.oam_data = data,
            PpuRegs::Scroll => if ppu.is_ready() { ppu.regs.scroll = data },
            PpuRegs::PpuAddr => if ppu.is_ready() { ppu.regs.addr = data },
            PpuRegs::PpuData => ppu.regs.data = data,
        };
    }

    pub fn write_oamdma(&mut self, data: u8) {
        let mut ppu = self.ppu.borrow_mut();
        // TODO: 要実装！ここに書きこんだ後にDMA転送が始まる。
        ppu.regs.oam_dma = data;
    }

    /// CPUからの、メモリを介したPPUからの読み込み要請
    pub fn read(&mut self, reg_type: PpuRegs) -> u8 {
        let ppu = self.ppu.borrow_mut();
        // 可能であればレジスタを読み込む。読み込み禁止の場合は、代わりにラッチの値を返す。
        let data = match reg_type {
            PpuRegs::Ctrl => self.latch,
            PpuRegs::Mask => self.latch,
            PpuRegs::Status => ppu.regs.status,
            PpuRegs::OamAddr => self.latch,
            PpuRegs::OamData => ppu.regs.oam_data,
            PpuRegs::Scroll => self.latch,
            PpuRegs::PpuAddr => self.latch,
            PpuRegs::PpuData => ppu.regs.data,
        };
        // バスを介した読み込みを行うと、ラッチも必ず更新される。
        self.latch = data;
        data
    }
    
    pub fn read_oamdma(&mut self) -> u8 {
        let ppu = self.ppu.borrow();
        // データバスを介さないので、レジスタの値をそのまま返す。
        ppu.regs.oam_dma
    }
}
