//! PPUの内部状態

use super::{Ppu, WARM_UP_TIME};

pub type FnState = fn(&mut Ppu);

/// 一時的な状態保持用
#[derive(Default)]
pub struct TmpState {
    pub counter: u8,
}

impl Ppu {
    /// 起動後、所定クロック経過するまでの状態
    pub fn prepare_step(&mut self) {
        if self.clock_counter >= WARM_UP_TIME {
            self.state = Default::default();
            self.fn_step = Ppu::exec_step;
        }
    }

    /// 実行準備が整った状態
    pub fn exec_step(&mut self) {
        
    }
}