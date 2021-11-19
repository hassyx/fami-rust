//! CPUの状態遷移

use crate::nes::cpu::Cpu;

/////////////////////////////////////
/// CPUの状態を管理する
pub struct StateManager {
    current: Box<dyn CpuState>,
}

impl Default for StateManager {
    fn default() -> Self {
        Self { current: Box::new(InitialState::default()) }
    }
}

impl StateManager {
    pub fn step(&mut self, cpu: &mut Cpu) {
        if let Some(new_state) = self.current.step(cpu) {
            self.next(new_state);
        }
    }

    pub fn next(&mut self, state: Box<dyn CpuState>) {
        self.current = state;
    }
}

/////////////////////////////////////
/// CPUの状態
pub trait CpuState {
    fn step(&mut self, cpu: &mut Cpu) -> Option<Box<dyn CpuState>>;
}

/////////////////////////////////////

/// 起動状態
#[derive(Default)]
pub struct InitialState {
    my_cnt: u64,
}
impl CpuState for InitialState {
    fn step(&mut self, cpu: &mut Cpu) -> Option<Box<dyn CpuState>> {
        self.my_cnt += 1;
        if self.my_cnt >= 6 {
            Some(Box::new(InitialState::default()))
        } else {
            None
        }
    }
}

/// 通常運転時
#[derive(Default)]
pub struct NormalState {}
impl CpuState for NormalState {
    fn step(&mut self, cpu: &mut Cpu) -> Option<Box<dyn CpuState>> {
        // 命令解析その他を実行…
        None
    }
}