//! CPUとPPUを繋ぐデータバス。

use num_derive::FromPrimitive;

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

/// CPUからPPUへアクセスする唯一の経路。
pub trait PpuDataBus {
    fn write(&mut self, reg_type: PpuRegs, data: u8);
    fn read(&mut self, reg_type: PpuRegs) -> u8;
    fn dma_write(&mut self, data: u8);
}
