//! PPUの内部状態

use super::{Ppu, WARM_UP_TIME};
use crate::nes::ppu_databus::*;

pub struct PpuState {
    pub step: fn(&mut Ppu),
    pub write: fn(&mut Ppu, PpuRegs, u8),
    pub read: fn(&mut Ppu, PpuRegs) -> u8,
}

pub const STATE_IDLING: PpuState = PpuState {
    step: Ppu::step_idling,
    write: Ppu::write_idling,
    read: Ppu::read_idling,
};

pub const STATE_READY: PpuState = PpuState {
    step: Ppu::step_ready,
    write: Ppu::write_ready,
    read: Ppu::read_ready,
};

impl Ppu {
    pub fn step_idling(&mut self) {
        if self.clock_counter > WARM_UP_TIME {
            self.state = &STATE_READY;
        }
    }

    pub fn step_ready(&mut self) {
        
    }

    /// 起動直後のPPUレジスタへの書き込み。
    /// PPUCTRL, PPUMASK, PPUSCROLL, PPUADDR への書き込みは無視される。
    pub fn write_idling(&mut self, reg_type: PpuRegs, data: u8) {
        // バスを介した書き込みを行うと、ラッチも必ず更新される。
        self.regs.databus = data;
        // PPUのレジスタへの値の設定、かつミラー領域への反映
        match reg_type {
            PpuRegs::Status => (), // PPUSTATUSは読み込み専用
            PpuRegs::OamAddr => self.regs.oam_addr = data,
            PpuRegs::OamData => self.regs.oam_data = data,
            PpuRegs::PpuAddr => (),
            PpuRegs::Ctrl |
            PpuRegs::Mask |
            PpuRegs::Scroll |
            PpuRegs::PpuData => (),
        };
    }

    /// PPUのレジスタへの書き込み。
    /// 全てのレジスタへの書き込みは正常に動作する。
    pub fn write_ready(&mut self, reg_type: PpuRegs, data: u8) {
        // バスを介した書き込みを行うと、ラッチも必ず更新される。
        self.regs.databus = data;
        // PPUのレジスタへの値の設定、かつミラー領域への反映
        match reg_type {
            PpuRegs::Ctrl => self.regs.ctrl = data,
            PpuRegs::Mask => self.regs.mask = data,
            PpuRegs::Status => (), // PPUSTATUSは読み込み専用
            PpuRegs::OamAddr => self.regs.oam_addr = data,
            PpuRegs::OamData => self.regs.oam_data = data,
            PpuRegs::Scroll => self.regs.scroll = data,
            PpuRegs::PpuAddr => self.regs.addr = data,
            PpuRegs::PpuData => self.regs.data = data,
        };
    }

    /// 起動直後のPPUレジスタからの読み込み。   
    pub fn read_idling(&mut self, reg_type: PpuRegs) -> u8 {
        // 可能であればレジスタを読み込む。その際ラッチも更新される。
        // 読み込み禁止レジスタの場合は、代わりに現在のラッチの値を返す。
        self.regs.databus = match reg_type {
            PpuRegs::Ctrl => self.regs.databus,
            PpuRegs::Mask => self.regs.databus,
            PpuRegs::Status => self.regs.status,
            PpuRegs::OamAddr => self.regs.databus,
            PpuRegs::OamData => self.regs.oam_data,
            PpuRegs::Scroll => self.regs.databus,
            PpuRegs::PpuAddr => self.regs.databus,
            PpuRegs::PpuData => self.regs.data,
        };
        self.regs.databus
    }

    pub fn read_ready(&mut self, reg_type: PpuRegs) -> u8 {
        self.regs.databus = match reg_type {
            PpuRegs::Ctrl => self.regs.databus,
            PpuRegs::Mask => self.regs.databus,
            PpuRegs::Status => self.regs.read_status(),
            PpuRegs::OamAddr => self.regs.databus,
            PpuRegs::OamData => self.regs.oam_data,
            PpuRegs::Scroll => self.regs.databus,
            PpuRegs::PpuAddr => self.regs.databus,
            PpuRegs::PpuData => self.regs.data,
        };
        self.regs.databus
    }
}
