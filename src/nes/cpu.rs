//! Emulated 6502.

use std::default;

use piston_window::draw_state::Stencil;

use crate::nes::rom;
use crate::nes::mem;

use super::cpu_state::InitialState;

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
    ram: Box<mem::MemCon>,
    clock_freq: u32,
    clock_cycle: f32,
    /// 起動後、リセットまたは電源断まで増加し続けるカウンター
    clock_counter: u64,
    /// 特定の処理が発動した場合などに利用される、一時的なカウンター
    tmp_counter: u64,
    regs: Registers,
    interruption: IntType,
    clock_count: u64,
    /// CPUの状態ごとに切り替わる関数。いわゆるStateパターンを実現するための仕組み。
    /// CPUのメインループ内で呼ばれる処理では、可能な限り動的なメモリ確保を避けたいため、
    /// 構造体ではなく関数ポインタで実現している。(動的な状態はCpu構造体の方に持たせている)
    fn_step: fn(&mut Cpu),
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

// ステータスフラグ
const F_CARRY: u8       = 0b0000_0001;
const F_ZERO: u8        = 0b0000_0010;
const F_INTERRUPT: u8   = 0b0000_0100;
const F_DECIMAL: u8     = 0b0000_1000;
const F_BREAK: u8       = 0b0001_0000;
const F_RESERVED: u8    = 0b0010_0000;
const F_OVERFLOW: u8    = 0b0100_0000;
const F_NETIVE: u8      = 0b1000_0000;

// 割り込みハンドラのアドレス:
const INT_NMI: u16        = 0xFFFA;
const INT_RESET: u16      = 0xFFFC;
const INT_IRQ: u16        = 0xFFFE;

/// Type of interruption.
#[derive(PartialEq, Clone, Copy)]
enum IntType {
    None,
    Reset,
    Nmi,
    Irq,
    Brk,
}

// 呼び出し元にwaitを入れるクロック数を返す実装も考えたが、
// 例え待つ必要がなくても「待つ必要があるかどうか」を判定する
// 余計な手間がかかるので止めた。

fn STATE_init(cpu: &mut Cpu) {
    debug_assert!(cpu.clock_count < 8);

    if cpu.clock_count == 6 {
        // clock 6, 7: Reset割り込みベクタテーブルを読み込む。
        // やってることは割り込みだが、起動時の特殊処理なのでスタックには手をつけない。
        let low = cpu.ram.read(0xFFFC, cpu.clock_count);
        let high = cpu.ram.read(0xFFFD, cpu.clock_count);
        // clock 8: 割り込みベクタを実行する(ここでは準備だけ)
        cpu.regs.pc = low as u16 | (high as u16) << 4;
    } else if cpu.clock_count == 7 {
        cpu.fn_step = STATE_normal;
    }
}

fn STATE_normal(cpu: &mut Cpu) {
    // PCで指しているメモリを1バイト読む

    // TODO: 6502の命令の判定方法を調べる。
}

impl Cpu {
    pub fn new(rom: &Box<rom::NesRom>, ram: Box<mem::MemCon>, clk_cnt: u64) -> Self {
        let mut my = Cpu {
            ram,
            clock_freq: CLOCK_FREQ_NTSC, // Use NTSC as default.
            clock_cycle: 1f32 / (CLOCK_FREQ_NTSC as f32),
            clock_counter: 0,
            tmp_counter: 0,
            interruption: IntType::None,
            regs: Registers::default(),
            clock_count: 0,
            fn_step: STATE_init,
        };

        {
            // PRG-ROM を RAM に展開
            let prg_rom = rom.prg_rom();
            let len = rom::PRG_ROM_UNIT_SIZE;
            if prg_rom.len() >= len {
                my.ram.raw_write(0x8000, &prg_rom[0..len], clk_cnt);
            }
            if prg_rom.len() >= (len * 2) {
                my.ram.raw_write(0xC000, &prg_rom[len..len*2], clk_cnt);
            } else {
                // PRG-ROMが2枚ない場合は、1枚目をコピーする。
                // TODO: MMCによってはPRG-ROMが2つ以上載っている可能性あり。
                my.ram.raw_write(0xC000, &prg_rom[0..len], clk_cnt);
            }
        }

        return my
    }

    /// メモリから命令を読み込んで実行。
    /// 命令自体は即座に実行されるが、戻り値として命令の実行完了に必要なクロック数を返す。
    /// エミュレーションの精度を上げたい場合は、呼び出し元でそのクロック数分、待機する。
    pub fn step(&mut self, clk_cnt: u64){
        self.clock_counter += 1;

        (self.fn_step)(self);   
        

        // 割り込み発生していた場合はそちらに移動
        if self.interruption != IntType::None {
            self.interrupt(self.interruption);
            self.interruption = IntType::None;
            // TODO: ここで返すのは interrupt() の戻り値でいいか？
        }

        // メモリから命令をフェッチ
        // そのためにはPCの値を知る必要がある。
        //self.mem.

        // 命令種別を判定
        // メモリとレジスタの状態を変更

        // 命令の完了に要するクロック数を返す
    }

    /// 割り込み
    fn interrupt(&self, int_type: IntType) {
        // TODO: PCの下位、上位、ステータスレジスタをスタックに積む
        // 次に、割り込みの種類ごとに決まったアドレスを読み込み、そこにジャンプする。

        // TODO: カセットのアドレスにアクセスする必要がある。

        // TODO: ステータスフラグの状態によっては、割り込みを無効にする必要がある。
    }
    
    /// 電源投入(リセット割り込み発生)
    pub fn power_on(&mut self, clk_cnt: u64) {
        // レジスタとメモリの初期化
        self.regs.a = 0;
        self.regs.x = 0;
        self.regs.y = 0;
        self.regs.s = 0xFD;
        //self.regs.p = 0x34;
        self.regs.p = F_INTERRUPT | F_BREAK | F_RESERVED;

        // 以下の初期値設定は、メモリが0クリアされているので厳密には意味がない。

        // APU状態のリセット
        // TODO: 厳密にはPPUのレジスタも書き換える必要がある(が、初期値0なので特に意味なし)
        self.ram.fill(0x4000..=0x400F, 0, clk_cnt);
        self.ram.fill(0x4010..=0x4013, 0, clk_cnt);
        self.ram.write(0x4015, 0, clk_cnt);
        self.ram.write(0x4017, 0, clk_cnt);

        // 物理RAMの初期化
        self.ram.fill(0x0000..=0x07FF, 0, clk_cnt);
    }

    pub fn reset(&mut self, ram: &mut mem::MemCon) {
        // リセット時にはRAMを初期化しない。初期化するのはゲーム側の仕事。
        self.regs.s -= 3;
        self.regs.p |= !F_INTERRUPT;
        // APUの初期化があるが省略
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


