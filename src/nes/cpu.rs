//! 6502 emulator.

use crate::nes::decoder;
use crate::nes::mem;
use crate::nes::rom;


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
    ram: Box<mem::MemCon>,
    clock_freq: u32,
    clock_cycle: f32,
    /// 起動後、リセットまたは電源断まで増加し続けるカウンター
    clock_counter: u64,
    /// 特定の処理が発動した場合などに利用される、一時的なカウンター
    tmp_counter: u64,
    regs: Registers,
    int: IntType,
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

/// Type of interruption.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IntType {
    None,
    Reset,
    Nmi,
    Irq,
    Brk,
}

/// 通常の実行処理
fn STATE_normal(cpu: &mut Cpu) {
    // 効率のよい命令デコードについてはここが詳しい。
    // https://llx.com/Neil/a2/opcodes.html
    cpu.execute();
}

/// 割り込み処理
fn STATE_interrupt(cpu: &mut Cpu) {
    debug_assert!(cpu.int_enabled());
    debug_assert!(cpu.tmp_counter < 8);
    debug_assert_ne!(cpu.int, IntType::None);

    //　割り込み処理は要7クロック。8クロック目に割り込みベクタの実行開始。

    if cpu.tmp_counter == 7 {
        // 割り込みを無効化
        cpu.flags_on(F_INT_DISABLE);
        // Brkフラグの設定
        if cpu.int == IntType::Brk {
            cpu.flags_on(F_BREAK);
        } else {
            cpu.flags_off(F_BREAK);
        }
        // Resetの場合はスタックを触らない
        if cpu.int != IntType::Reset {
            // clock 3,4: プログラムカウンタをHigh, Lowの順にpush
            // Brk命令は2バイトあり、ここに来た時点で1バイト目を読んでいるので、PCを更に+1。
            if cpu.int == IntType::Brk { cpu.regs.pc += 1 }
            cpu.push((cpu.regs.pc >> 8 & 0x00FF) as u8);
            cpu.push((cpu.regs.pc & 0x00FF) as u8);
            // clock 5: ステータスレジスタをpush
            cpu.push(cpu.regs.p);
        }
        // スタックに保存したあとは、無条件でBreakフラグを落とす(常に0)
        cpu.flags_off(F_BREAK);
        // clock 6,7: 割り込みベクタテーブルを読み込む。
        let vec_addr = match cpu.int {
            IntType::Reset => ADDR_INT_RESET,
            IntType::Nmi => ADDR_INT_NMI,
            IntType::Irq | IntType::Brk => ADDR_INT_IRQ,
            IntType::None => panic!("invalid IntType."),
        };
        let low = cpu.ram.read(vec_addr);
        let high = cpu.ram.read(vec_addr+1);
        // clock 8: 割り込みベクタを実行する(ここでは準備だけ)
        cpu.regs.pc = ((high as u16) << 4) | low as u16;
        cpu.int = IntType::None;
        cpu.switch_state(STATE_normal);
        return;
    }
}

impl Cpu {
    pub fn new(rom: &Box<rom::NesRom>, ram: Box<mem::MemCon>) -> Self {
        let mut my = Cpu {
            ram,
            clock_freq: CLOCK_FREQ_NTSC, // Use NTSC as default.
            clock_cycle: 1f32 / (CLOCK_FREQ_NTSC as f32),
            clock_counter: 0,
            tmp_counter: 0,
            int: IntType::None,
            regs: Registers::default(),
            clock_count: 0,
            fn_step: STATE_interrupt,
        };

        {
            // PRG-ROM を RAM に展開
            let prg_rom = rom.prg_rom();
            let len = rom::PRG_ROM_UNIT_SIZE;
            if prg_rom.len() >= len {
                my.ram.raw_write(0x8000, &prg_rom[0..len]);
            }
            if prg_rom.len() >= (len * 2) {
                my.ram.raw_write(0xC000, &prg_rom[len..len*2]);
            } else {
                // PRG-ROMが2枚ない場合は、1枚目をコピーする。
                // TODO: MMCによってはPRG-ROMが2つ以上載っている可能性あり。
                my.ram.raw_write(0xC000, &prg_rom[0..len]);
            }
        }

        return my
    }
    
    /// メモリから命令を読み込んで実行。
    /// 命令自体は即座に実行されるが、戻り値として命令の実行完了に必要なクロック数を返す。
    /// エミュレーションの精度を上げたい場合は、呼び出し元でそのクロック数分、待機する。
    /// 
    /// TODO: 外部からのクロックは不要？
    pub fn step(&mut self){
        self.clock_counter += 1;
        self.tmp_counter += 1;
        (self.fn_step)(self);
    }

    /// PCが指すメモリを1バイト読み、PCを1進める。
    pub fn fetch(&mut self) -> u8 {
        let data = self.ram.read(self.regs.pc);
        // TODO: PCがオーバーフローした場合の挙動は？
        self.regs.pc += 1;
        data
    }

    /// 割り込みチェック
    pub fn check_int(&mut self) {
        // 割り込み発生時は現在実行中の命令を中断する
        if (self.int != IntType::None) {
            self.switch_state(STATE_interrupt);
        }
    }

    fn switch_state(&mut self, fn_next: fn(&mut Cpu)) {
        self.tmp_counter = 0;
        self.fn_step = fn_next;
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

        // 以下の初期値設定は、メモリが0クリアされているので厳密には意味がない。

        // APU状態のリセット
        // TODO: 厳密にはPPUのレジスタも書き換える必要がある(が、初期値0なので特に意味なし)
        self.ram.raw_fill(0x4000..=0x400F, 0);
        self.ram.raw_fill(0x4010..=0x4013, 0);
        self.ram.write(0x4015, 0);
        self.ram.write(0x4017, 0);

        // 物理RAMの初期化
        self.ram.raw_fill(0x0000..=0x07FF, 0);

        // スタート時は直にReset割り込みから実行開始
        self.flags_off(F_INT_DISABLE);
        self.int = IntType::Reset;
        self.switch_state(STATE_interrupt);
    }

    pub fn reset(&mut self, ram: &mut mem::MemCon) {
        // リセット時にはRAMを初期化しない。初期化するのはゲーム側の仕事。
        self.regs.s -= 3;
        self.flags_on(F_INT_DISABLE);
    }

    pub fn interrupt(&mut self, int_type: IntType) {
        // !!!!!!!!!!
        // TODO: 割り込みは現在実行中の命令が完了するまで待ち、次の命令の実行直前にチェックすればよい。
        // ただし、BRK中の割り込みは特殊。だけどエミュレーターではそこまで実装しなくていい？
        // !!!!!!!!!!

        // 割り込み無効フラグが立っている場合、IRQとBRKの発生を無視する。
        if !self.int_enabled() &&
            (int_type == IntType::Irq || int_type == IntType::Brk)
        {
            return
        }
        self.int = int_type;
    }

    pub fn int_enabled(&self) -> bool {
        (self.regs.p & F_INT_DISABLE) == 0
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
        self.ram.write(addr, data);
        self.regs.s = self.regs.s.wrapping_sub(1);
    }

    pub fn pop(&mut self) -> u8 {
        self.check_stack_underflow();

        let addr = ADDR_STACK_UPPER & (self.regs.s as u16);
        let data = self.ram.read(addr);
        self.regs.s = self.regs.s.wrapping_add(1);
        data
    }

    /// 命令を読み込んで実行。戻り値として命令の実行に必要なクロックを返す。
    pub fn execute(&mut self) -> u8 {
        decoder::decode(self)
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


