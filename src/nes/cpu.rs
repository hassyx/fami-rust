//! 6502 emulator.

mod cpu_state;
mod decoder;
mod executer;

use crate::nes::mem;
use crate::nes::rom;
use crate::nes::cpu::cpu_state::*;

/// NTSC版のクロック周波数(Hz)
const CLOCK_FREQ_NTSC: u32 = 1789773;
/// PAL版のクロック周波数(Hz)
const CLOCK_FREQ_PAL: u32 = 1662607;

// ステータスフラグ
/// キャリー発生時に1。
const F_CARRY: u8       = 0b0000_0001;
/// 演算結果が0だった場合に1。
const F_ZERO: u8        = 0b0000_0010;
/// 割り込み禁止なら1。ただしNMIには影響しない。
const F_INT_DISABLE: u8   = 0b0000_0100;
/// 10進モードがONなら1。NESでは意味を持たない。
const F_DECIMAL: u8     = 0b0000_1000;
/// 割り込みがBRKだったら1。IRQとBRKの判別用。
/// このフラグは本来レジスタ上には存在しない。
/// ユーザーは、スタックにpushされた内容からフラグの値を判断する。
const F_BREAK: u8       = 0b0001_0000;
/// 予約領域。常に1。
const F_RESERVED: u8    = 0b0010_0000;
/// オーバーフロー。最上位ビットからの繰り下がり、
/// または最上位ビットへの繰り上がりが発生した場合に1になる。
const F_OVERFLOW: u8    = 0b0100_0000;
/// 演算結果が負だった場合に1。Aレジスタの最上位ビットと同じ。
const F_NETIVE: u8      = 0b1000_0000;

// 割り込みハンドラのアドレス:
const ADDR_INT_NMI: u16        = 0xFFFA;
const ADDR_INT_RESET: u16      = 0xFFFC;
const ADDR_INT_IRQ: u16        = 0xFFFE;

// スタックポインタの上位アドレス
const ADDR_STACK_UPPER: u16    = 0x0100;

/// 6502 (RICHO 2A03)
pub struct Cpu {
    mem: Box<mem::MemCon>,
    clock_freq: u32,
    clock_cycle: f32,
    /// 起動後、リセットまたは電源断まで増加し続けるカウンター
    clock_counter: u64,
    regs: Registers,
    /// リセットピン
    reset_trigger: bool,
    /// エッジトリガな割り込み検出機(NMI用)。
    nmi_trigger: bool,
    /// レベルトリガな割り込み検出機(IRQ用)。
    /// TODO: 正確なエミュレートのためには、トリガの有効期間を実装する必要がある。
    irq_trigger: bool,
    /// ソフトウェア割り込みの場合にtrue
    irq_is_brake: bool,
    /// 割り込み検出検出機のポーリング処理を停止
    int_polling_enabled: bool,
    /// CPUの状態ごとに切り替わる関数。いわゆるStateパターンを実現するための仕組み。
    /// こうした理由は、1クロックサイクルごとに走る条件判定処理をできるだけ減らしたかったのと、
    /// CPUのメインループ内で呼ばれる処理では、可能な限り動的なメモリ確保を避けたいため、
    /// 構造体ではなく関数ポインタで実現している。(動的な状態はCpu構造体の方に持たせている)
    fn_step: fn(&mut Cpu),
    state: TmpState,
    //state_exec: Box<StateExec>,
    //state_int: Box<StateInt>,
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
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IntType {
    None,
    Reset,
    Nmi,
    Irq,
    Brk,
}

impl Cpu {
    pub fn new(rom: &Box<rom::NesRom>, ram: Box<mem::MemCon>) -> Self {

        let mut my = Cpu {
            mem: ram,
            clock_freq: CLOCK_FREQ_NTSC, // Use NTSC as default.
            clock_cycle: 1f32 / (CLOCK_FREQ_NTSC as f32),
            clock_counter: 0,
            reset_trigger: false,
            nmi_trigger: false,
            irq_trigger: false,
            irq_is_brake: false,
            int_polling_enabled: false,
            regs: Registers::default(),
            fn_step: Cpu::int_step,
            state: TmpState::default(),
        };

        {
            // PRG-ROM を RAM に展開
            let prg_rom = rom.prg_rom();
            let len = rom::PRG_ROM_UNIT_SIZE;
            if prg_rom.len() >= len {
                my.mem.raw_write(0x8000, &prg_rom[0..len]);
            }
            if prg_rom.len() >= (len * 2) {
                my.mem.raw_write(0xC000, &prg_rom[len..len*2]);
            } else {
                // PRG-ROMが2枚ない場合は、1枚目をコピーする。
                // TODO: MMCによってはPRG-ROMが2つ以上載っている可能性あり。
                my.mem.raw_write(0xC000, &prg_rom[0..len]);
            }
        }

        return my
    }
    
    /// 電源投入(リセット割り込み発生)
    pub fn power_on(&mut self) {
        // レジスタとメモリの初期化
        self.regs.a = 0;
        self.regs.x = 0;
        self.regs.y = 0;
        self.regs.s = 0xFD;
        //self.regs.p = 0x34;
        self.flags_on(F_INT_DISABLE | F_BREAK | F_RESERVED);

        // APU状態のリセット
        // TODO: 厳密にはPPUのレジスタも書き換える必要がある(が、初期値0なので特に意味なし)
        self.mem.raw_fill(0x4000..=0x400F, 0);
        self.mem.raw_fill(0x4010..=0x4013, 0);
        self.mem.write(0x4015, 0);
        self.mem.write(0x4017, 0);

        // 物理RAMの初期化
        self.mem.raw_fill(0x0000..=0x07FF, 0);

        // 割り込み状態の初期化
        self.clear_all_int_trigger();

        // スタート時は直にReset割り込みから実行開始        
        self.reset_trigger = true;
        self.switch_state_int();
    }

    pub fn reset(&mut self, ram: &mut mem::MemCon) {
        // リセット時にはRAMを初期化しない。初期化するのはゲーム側の仕事。
        self.regs.s -= 3;
        self.flags_on(F_INT_DISABLE);
    }

    /// 1クロックサイクル進める。
    pub fn step(&mut self){
        self.clock_counter += 1;
        self.state.counter += 1;
        (self.fn_step)(self);
        self.check_int();
    }

    fn check_int(&mut self) {
        if !self.int_polling_enabled {
            return
        }
        if self.reset_trigger || self.nmi_trigger ||
            (!self.int_disabled() && self.irq_trigger) {
            // 割り込みが発生しているなら、割り込みモードへ遷移。
            self.switch_state_int();
        }
    }

    fn clear_all_int_trigger(&mut self) {
        self.reset_trigger = false;
        self.nmi_trigger = false;
        self.irq_trigger = false;
        self.irq_is_brake = false;
    }

    /// PCが指すメモリを1バイト読み、PCを1進める。
    pub fn fetch(&mut self) -> u8 {
        let data = self.mem.read(self.regs.pc);
        // TODO: PCがオーバーフローした場合の挙動は？
        self.regs.pc += 1;
        data
    }

    fn switch_state_int(&mut self) {
        self.state = TmpState::default();
        self.fn_step = Cpu::int_step;
    }

    fn switch_state_exec(&mut self) {
        self.state = TmpState::default();
        self.fn_step = Cpu::exec_step;
    }

    fn exec_finished(&mut self) {
        self.state = TmpState::default();
        self.int_polling_enabled = true;
    }

    /*
    // TODO: 割り込み処理はエッジトリガ・レベルトリガ等の厳密な対応が必要。
    // ひとまず現状は、発生した割り込みを保存し続ける実装とする。
    pub fn interrupt(&mut self) {
        // 割り込み無効フラグが立っている場合、IRQとBRKの発生を無視する。
        if self.int_disabled() &&
            (int_type == IntType::Irq || int_type == IntType::Brk)
        {
            return
        }
        
        match int_type {
            IntType::Reset => {
                self.reset_trigger = true;
                self.is_brake = false;
            }
            IntType::Nmi => {
                self.nmi_trigger = true;
                self.is_brake = false;
            }
            IntType::Irq => {
                self.nmi_trigger = true;
                self.is_brake = false;
            },
            IntType::Brk => {
                self.nmi_trigger = true;
                self.is_brake = true;
            },
        }
    }
    */

    pub fn int_disabled(&self) -> bool {
        (self.regs.p & F_INT_DISABLE) != 0
    }

    pub fn flags_on(&mut self, flags: u8) {
        self.regs.p |= flags;
    }

    pub fn flags_off(&mut self, flags: u8) {
        self.regs.p &= !flags;
    }

    pub fn push(&mut self, data: u8) {
        self.check_stack_overflow();

        let addr = ADDR_STACK_UPPER & (self.regs.s as u16);
        self.mem.write(addr, data);
        self.regs.s = self.regs.s.wrapping_sub(1);
    }

    pub fn pop(&mut self) -> u8 {
        self.check_stack_underflow();

        let addr = ADDR_STACK_UPPER & (self.regs.s as u16);
        let data = self.mem.read(addr);
        self.regs.s = self.regs.s.wrapping_add(1);
        data
    }

    #[cfg(debug_assertions)]
    fn check_stack_overflow(&self) {
        if self.regs.s <= 0 {
            println!("stack overflow detected.");
        }
    }

    #[cfg(debug_assertions)]
    fn check_stack_underflow(&self) {
        if self.regs.s >= u8::MAX {
            println!("stack underflow detected.");
        }
    }

    /*
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
    */
}


