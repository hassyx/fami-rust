//! PPUの内部状態

use super::Ppu;

pub type FnState = fn(&mut Ppu);

/// 一時的な状態保持用
#[derive(Default)]
pub struct TmpState {
    pub counter: u8,
}

impl Ppu {
    /// 起動後、所定クロック経過するまでの状態
    pub fn prepare_step(&mut self) {
        // 何もしない
    }
}