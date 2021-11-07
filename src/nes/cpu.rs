//! Emulated 6502.

use crate::nes::rom;
use crate::nes::ppu;

// CPUは外部からは state machine として見えるべき。
// クロックも設定できるようにする。実行時であっても変えられる。

// コンストラクタで全てを設定すべきなのか、それともプロパティ的なもので
// アクセスさせるべきなのか。Rustについてもっと調べること。

pub const CLOCK_FREQ_NTSC: usize = 1789773;
pub const CLOCK_FREQ_PAL: usize = 1662607;
pub const RAM_SIZE: usize = 16384;

/// 6502 (RICHO 2A03)
pub struct CPU {
    ram: Vec<u8>,
    rom: Option<Box<rom::NesRom>>,
    //ppu: ppu::PPU,
    clock_freq: usize,
    clock_cycle: f32,
    //regs: Registers,
}

struct Registers {
    /// Accumulator
    A: u8,
    /// Index Regeister 1
    X: u8,
    /// Index Regeister 2
    Y: u8,
    /// Stack Pointer
    S: u8,
    /// Status Flag
    P: u8,
    /// Program Counter
    PC: u16,
}

/// Type of interruption.
enum IntType {
    Reset,
    Nmi,
    Irq,
    Brk,
}

impl Default for CPU {
    fn default() -> Self {
        Self {
            ram: vec!(0; RAM_SIZE),
            rom: Option::None,
            clock_freq: CLOCK_FREQ_NTSC, // Use NTSC as default.
            clock_cycle: 1f32 / (CLOCK_FREQ_NTSC as f32),
        }
    }
}

impl CPU {
    /*
    fn new() -> Self {
        
    }
    */

    /// 電源投入(リセット割り込み発生)
    pub fn power_on(&self) {
        std::thread::spawn(CPU::main_loop);
        self.interrupt(IntType::Reset);
    }

    pub fn reset(&self) {
        // TODO: 電源投入時とはリセットする値がちょっと違う
    }

    pub fn attach_rom(&mut self, rom: Box<rom::NesRom>) {
        self.rom = Option::Some(rom);
    }

    /// clock で指定したクロック数分 wait を入れる。
    pub fn wait(&self, clock: u32) {
        // TODO: コンパイルオプションで (無効/有効/任意の値設定) に対応する。
        // TODO: イベントループで待つ必要がある。
    }

    fn main_loop() {

    }

    /*
    /// RAM領域を生成する。
    fn create_ram_area(&mut self, size: usize) {
        self.ram = Option::Some(vec![0; size]);
    }
    */



    /// クロック周波数(clock_freq)を設定。
    /// 同時に clock_cycle も更新する。
    fn set_clock_freq(&mut self, clock: usize) {
        self.clock_freq = clock;
        self.clock_cycle = 1f32 / (self.clock_freq as f32);
    }

    /// 割り込み
    fn interrupt(&self, int_type: IntType) {

    }
}


