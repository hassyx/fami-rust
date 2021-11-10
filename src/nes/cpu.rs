//! Emulated 6502.

use std::rc::Rc;
use piston_window::PressEvent;

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
pub struct CPU<'a> {
    rom: &'a rom::NesRom,
    ram: Box<mem::MemCon<'a>>,
    clock_freq: u32,
    clock_cycle: f32,
    regs: Registers,
    interruption: IntType,
}

#[derive(Default)]
pub struct Registers {
    /// Accumulator
    pub a: u8,
    /// Index Regeister 1
    pub x: u8,
    /// Index Regeister 2
    pub y: u8,
    /// Stack Pointer
    pub s: u8,
    /// Status Flag
    pub p: u8,
    /// Program Counter
    pub pc: u16,
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

fn mem_callback(addr: usize) {

}

impl<'a> CPU<'a> {
    pub fn new(cpu_regs: &'a Registers, ppu_regs: &'a ppu::Registers, rom: &'a rom::NesRom) -> Self {
        //let regs = Box::new(Registers::default());
        let mut my = CPU {
            rom,
            ram: Box::new(mem::MemCon::new(cpu_regs, ppu_regs)),
            clock_freq: CLOCK_FREQ_NTSC, // Use NTSC as default.
            clock_cycle: 1f32 / (CLOCK_FREQ_NTSC as f32),
            interruption: IntType::None,
            regs: Registers::default(),
        };

        {
            // PRG-ROM を RAM に展開
            // TODO: PRG-ROMが2枚ない場合のメモリへの反映方法
            let prg_rom = my.rom.prg_rom();
            let len = rom::PRG_ROM_UNIT_SIZE;
            if prg_rom.len() >= len {
                my.ram.raw_write(0x8000, &prg_rom[0..len]);
            }
            if prg_rom.len() >= (len * 2) {
                my.ram.raw_write(0xC000, &prg_rom[len..len*2]);
            }
        }

        return my
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
        // そのためにはPCの値を知る必要がある。
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
    pub fn power_on(&mut self) {
        // レジスタとメモリの初期化
        //self.regs.p = 0x34;


        // TODO: RAMに展開する！

        self.interruption = IntType::Reset;
    }

    pub fn reset(&self) {
        // TODO: 電源投入時とはリセットする値がちょっと違う
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


