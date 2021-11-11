//! Emulated 6502.

use std::io::BufRead;

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
pub struct Cpu {
    clock_freq: u32,
    clock_cycle: f32,
    regs: Registers,
    interruption: IntType,
    clock_count: u64,
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

// ステータスフラグs
const F_CARRY: u8       = 0b0000_0001;
const F_ZERO: u8        = 0b0000_0010;
const F_INTERRUPT: u8   = 0b0000_0100;
const F_DECIMAL: u8     = 0b0000_1000;
const F_BREAK: u8       = 0b0001_0000;
const F_RESERVED: u8    = 0b0010_0000;
const F_OVERFLOW: u8    = 0b0100_0000;
const F_NETIVE: u8      = 0b1000_0000;

/// Type of interruption.
#[derive(PartialEq, Clone, Copy)]
enum IntType {
    None,
    Reset,
    Nmi,
    Irq,
    Brk,
}

impl Cpu {
    pub fn new(rom: &Box<rom::NesRom>, ram: &mut mem::MemCon) -> Self {
        let my = Cpu {
            clock_freq: CLOCK_FREQ_NTSC, // Use NTSC as default.
            clock_cycle: 1f32 / (CLOCK_FREQ_NTSC as f32),
            interruption: IntType::None,
            regs: Registers::default(),
            clock_count: 0,
        };

        {
            // PRG-ROM を RAM に展開
            // TODO: PRG-ROMが2枚ない場合のメモリへの反映方法
            let prg_rom = rom.prg_rom();
            let len = rom::PRG_ROM_UNIT_SIZE;
            if prg_rom.len() >= len {
                ram.raw_write(0x8000, &prg_rom[0..len]);
            }
            if prg_rom.len() >= (len * 2) {
                ram.raw_write(0xC000, &prg_rom[len..len*2]);
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

        // TODO: ステータスフラグの状態によっては、割り込みを無効にする必要がある。
    }
    
    /// 電源投入(リセット割り込み発生)
    pub fn power_on(&mut self, ram: &mut mem::MemCon) {
        // レジスタとメモリの初期化
        self.regs.a = 0;
        self.regs.x = 0;
        self.regs.y = 0;
        self.regs.s = 0xFD;
        //self.regs.p = 0x34;
        self.regs.p = F_INTERRUPT | F_BREAK | F_RESERVED;
        
        enum Flags {
            Carry       = 0b0000_0001,
            Zero        = 0b0000_0010,
            Interrupt   = 0b0000_0100,
            Decimal     = 0b0000_1000,
            Break       = 0b0001_0000,
            Reserved    = 0b0010_0000,
            Overflow    = 0b0100_0000,
            Negative    = 0b1000_0000,
        }

        // APU状態のリセット
        ram.fill(0x4000..0x400F, 0);
        ram.fill(0x4010..0x4013, 0);
        ram.write_b(0x4015, 0);
        ram.write_b(0x4017, 0);

        // 物理RAMの初期化
        ram.fill(0x0000..0x07FF, 0);
        
        // TODO: ROMの内容をRAMに展開する！

        self.interruption = IntType::Reset;
    }

    pub fn reset(&mut self) {
        // TODO: 起動時にも、レジスタの初期化処理が走るのか？

        // リセット時にはRAMを初期化しない。初期化するのはゲーム側の仕事。
        self.regs.s -= 3;
        self.regs.p |= !F_INTERRUPT;
        // APUの初期化があるが省略
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


