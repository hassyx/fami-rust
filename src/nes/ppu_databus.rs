//! CPUとPPUを繋ぐデータバス。

use std::cell::RefCell;
use std::rc::Rc;

use crate::nes::ppu::Ppu;

pub struct DataBus {
    ppu: Rc<RefCell<Ppu>>,
    latch: u8,
}

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

impl DataBus {

    pub fn new(ppu: Rc<RefCell<Ppu>>) -> Self {
        Self {
            ppu,
            latch: 0,
        }
    }

    pub fn power_on(&mut self) {
        
    }

    pub fn exec(&mut self) {

    }

    pub fn write(&mut self, ppu_reg: PpuRegs, data: u8) {

    }

    pub fn read(&mut self, ppu_reg: PpuRegs) -> u8 {
        0
    }
}
