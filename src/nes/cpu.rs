//! Emulated 6502.

use std::rc::Rc;
use crate::nes::rom;
use crate::nes::mem;
use crate::nes::ppu;

// CPUは外部からは state machine として見えるべき。
// クロックも設定できるようにする。実行時であっても変えられる。

// コンストラクタで全てを設定すべきなのか、それともプロパティ的なもので
// アクセスさせるべきなのか。Rustについてもっと調べること。

/// NTSC版のクロック周波数(Hz)
pub const CLOCK_FREQ_NTSC: u32 = 1789773;
/// PAL版のクロック周波数(Hz)
pub const CLOCK_FREQ_PAL: u32 = 1662607;

/// 6502 (RICHO 2A03)
pub struct CPU {
    ram: Box<mem::MemCon>,
    rom: Option<Box<rom::NesRom>>,
    clock_freq: u32,
    clock_cycle: f32,
    regs: Rc<Registers>,
    interruption: IntType,
}

#[derive(Default)]
pub struct Registers {
    /// Accumulator
    a: u8,
    /// Index Regeister 1
    x: u8,
    /// Index Regeister 2
    y: u8,
    /// Stack Pointer
    s: u8,
    /// Status Flag
    p: u8,
    /// Program Counter
    pc: u16,
}

/// Type of interruption.
#[derive(PartialEq, Clone, Copy)]
pub enum IntType {
    None,
    Reset,
    Nmi,
    Irq,
    Brk,
}

impl CPU {

    pub fn new(ppu_regs: Rc<ppu::Registers>) -> Self {
        let regs = Rc::new(Registers::default());
        Self {
            ram: Box::new(mem::MemCon::new(
                Rc::clone(&regs),
                ppu_regs,
            )),
            rom: Option::None,
            clock_freq: CLOCK_FREQ_NTSC, // Use NTSC as default.
            clock_cycle: 1f32 / (CLOCK_FREQ_NTSC as f32),
            interruption: IntType::None,
            regs: Rc::clone(&regs),
        }
    }

    /// メモリから命令を読み込んで実行。
    /// 命令自体は即座に実行されるが、戻り値として命令の実行完了に必要なクロック数を返す。
    /// エミュレーションの精度を上げたい場合は、呼び出し元でそのクロック数分、待機する。
    pub fn exec(&mut self) -> u32 {
        // 割り込み発生していた場合はそちらに移動
        if self.interruption != IntType::None {
            self.interrupt(self.interruption);
            self.interruption = IntType::None;
            // TODO: ここで返すのは interrupt() の戻り値でいいか？
            return 10;
        }

        // メモリから命令をフェッチ
        // そのためにはPCの値
        //self.mem.

        // 命令種別を判定
        // メモリとレジスタの状態を変更

        // 命令の完了に要するクロック数を返す
        10
    }

    /// 割り込み
    fn interrupt(&self, int_type: IntType) {
        // TODO: PCの下位、上位、ステータスレジスタをスタックに積む
        // 次に、割り込みの種類ごとに決まったアドレスを読み込み、そこにジャンプする。

        // TODO: カセットのアドレスにアクセスする必要がある。

        
    }
    
    /// 電源投入(リセット割り込み発生)
    pub fn power_on(&self) {
        self.interrupt(IntType::Reset);
    }

    pub fn reset(&self) {
        // TODO: 電源投入時とはリセットする値がちょっと違う
    }

    pub fn attach_rom(&mut self, rom: Box<rom::NesRom>) {
        // TODO: おそらく、romを保存する必要はない。
        // ここにromが渡ってきた時点で、フラグに沿ってRAM空間に
        // データを展開すればよい。
        self.rom = Option::Some(rom);
    }

    /// clock で指定したクロック数分 wait を入れる。
    pub fn wait(&self, clock: u32) {
        // TODO: コンパイルオプションで (無効/有効/任意の値設定) に対応する。
        // TODO: イベントループで待つ必要がある。
    }

    /// クロック周波数(clock_freq)を設定。
    /// 同時に clock_cycle も更新する。
    fn set_clock_freq(&mut self, clock: u32) {
        self.clock_freq = clock;
        self.clock_cycle = 1f32 / (self.clock_freq as f32);
    }

    fn clock_freq(&self) -> u32 {
        self.clock_freq
    }

    fn clock_cycle(&self) -> f32 {
        self.clock_cycle
    }
}


