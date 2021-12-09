//! 6502 emulator.

#[macro_use] mod macros;
mod cpu_state;
mod decoder;
mod executer;
mod exec_core_g1;
mod exec_core_g2;
mod exec_core_g3;

use bitflags::bitflags;

use crate::nes::mem;
use crate::nes::rom;
use crate::nes::cpu::cpu_state::*;

/// NTSC版のクロック周波数(Hz)
const CLOCK_FREQ_NTSC: u32 = 1789773;
/// PAL版のクロック周波数(Hz)
const CLOCK_FREQ_PAL: u32 = 1662607;

// スタックポインタの上位アドレス
const ADDR_STACK_UPPER: u16 = 0x0100;


bitflags! {
    /// ステータスフラグ
    pub struct Flags: u8 {
        /// 加算でキャリーが、または減算でボローが発生した時に1。
        const CARRY       = 0b0000_0001;
        /// 演算結果が0だった場合に1。
        const ZERO        = 0b0000_0010;
        /// 割り込み禁止なら1。ただしNMIには影響しない。
        const INT_DISABLE = 0b0000_0100;
        /// 10進モードがONなら1。NESでは意味を持たない。
        const DECIMAL     = 0b0000_1000;
        /// 割り込みがBRKだったら1。IRQとBRKの判別用。
        /// このフラグは本来レジスタ上には存在しない。
        /// ユーザーは、スタックにpushされたPレジスタの内容から、フラグの値を判断する。
        const BREAK       = 0b0001_0000;
        /// 予約領域。常に1。
        const RESERVED    = 0b0010_0000;
        /// オーバーフロー。最上位ビットからの繰り下がり、
        /// または最上位ビットへの繰り上がりが発生した場合に1になる。
        const OVERFLOW    = 0b0100_0000;
        /// 演算結果が負だった場合に1。Aレジスタの最上位ビットと同じ。
        const NEGATIVE    = 0b1000_0000;
    }
}

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

impl Registers {
    pub fn int_disabled(&self) -> bool {
        (self.p & Flags::INT_DISABLE.bits) != 0
    }

    pub fn flags_on(&mut self, flags: Flags) {
        self.p |= flags.bits;
    }

    pub fn flags_off(&mut self, flags: Flags) {
        self.p &= !flags.bits;
    }

    /// valのMSBが1ならNegativeフラグをon、0ならoff。
    pub fn change_negative_by_value(&mut self, val: u8) {
        let z_flag: u8 = val & Flags::NEGATIVE.bits;
        self.p = (self.p & !Flags::NEGATIVE.bits) | z_flag;
    }

    /// valの値が0ならZeroフラグをon、それ以内ならoff。
    pub fn change_zero_by_value(&mut self, val: u8) {
        let z_flag: u8 = ((val == 0) as u8) << 1;
        self.p = (self.p & !Flags::ZERO.bits) | z_flag;
    }

    fn add_with_carry(val1: u8, val2: u8, carry: bool) -> (u8, bool) {
        // 全ての値をu8から16bitに拡張した上で加算を行う。
        // u8が取りうる最大の値が、Carry含めて加算された場合の結果は、以下の通り。
        //
        // $FF + $FF + 1 = $1FE(=0b0000_0001_1111_1111)
        //
        // つまりキャリー含めて全て加算しても結果は $1FF となり、
        // 先頭ビットの値をそのままCarryフラグとして利用できる。
        let result: u16 = (val1 as u16) + (val2 as u16) + (carry as u16);
        let new_carry = (result & 0x0100) != 0;
        // キャリーは記録したので上位8bitは削っていい
        let result = result as u8;
        // 2バイトなのでレジスタ経由で渡してくれるはず...
        (result, new_carry)
    }

    pub fn a_add(&mut self, val: u8) {
        let (result, carry) = 
            Self::add_with_carry(self.a, val, (self.p & Flags::CARRY.bits) != 0);

        // 桁溢れが発生していたらCarryをOn。そうでなければクリア。
        self.p = (self.p & !Flags::CARRY.bits) | carry as u8;
        // 演算結果のMSBが 0 から 1 に「変わった」場合にのみ、Overflowフラグを立てる。
        // そうでない場合は、例え結果のMSBが 1 でも、Overflowフラグをクリアする。
        // 加算する数値を(M, N)とした場合、"(M^result) & (N^result) & 0x80 != 0" で判定可能。
        // 詳細: http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
        self.p = {
            let overflowed = ((self.a ^ result) & (val ^ result) & 0x80) != 0;
            let overflow_bit = (overflowed as u8) << 6;
            (self.p & !Flags::OVERFLOW.bits) | overflow_bit
        };
        // 演算結果のMSBが 1 なら、ZeroをOn。そうでなければクリア。
        self.change_negative_by_value(result);
        // 演算結果が 0 なら、ZeroをOn。そうでなければクリア。
        self.change_zero_by_value(result);

        self.a = result;
    }

    pub fn a_sub(&mut self, val: u8) {
        // Carryフラグの扱いについて:
        // 6502は単純化のため、加算と減算で同じ演算機を利用している。
        // よってフラグの設定やその意味も、ADCそれに準ずる。
        // 具体的には、レジスタAに対し「1の補数の加算」を行った結果、
        // 桁溢れが発生した場合にCarryがOn、桁溢れが起きなかった場合にOffとなる。
        //
        // つまり、レジスタAと減算する値(の1の補数)を加算して、
        // 桁溢れした8bit目の値をそのままCarryフラグの値に利用できる。
        // 
        // この単純化のため、6502は減算時に「Borrowが発生した場合にCarryがOff、そうでない場合にOn」
        // という、直感に反するフラグ設定が行われる。また、Borrowの影響を無視して真っさらな状態で減算を行うには、
        // 「まずSECでCarry(=Borrow)フラグを"立てる"」という、これまた変なルールが生まれてしまう。
        
        // 上記より、SBCは、減算する値を1の補数で表したADCに等しい。
        self.a_add(!val);
    }

    pub fn a_cmp(&mut self, val: u8) {
        // CMP = Aに対して2の補数を加算し、加算の際にCarryを考慮せず、計算後にOverflowが変化しないADC。
        let (result, carry) = 
            Self::add_with_carry(self.a, val.wrapping_neg(), false);

        // 桁溢れが発生していたらCarryをOn。そうでなければクリア。
        self.p = (self.p & !Flags::CARRY.bits) | carry as u8;
        // 演算結果のMSBが 1 なら、ZeroをOn。そうでなければクリア。
        self.change_negative_by_value(result);
        // 演算結果が 0 なら、ZeroをOn。そうでなければクリア。
        self.change_zero_by_value(result);
    }
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
        self.regs.flags_on(Flags::INT_DISABLE | Flags::BREAK | Flags::RESERVED);

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
        self.regs.flags_on(Flags::INT_DISABLE);
    }

    /// 1クロックサイクル進める。
    pub fn step(&mut self){
        self.clock_counter += 1;
        self.state.counter += 1;
        (self.fn_step)(self);
        self.check_int();

        print_cpu_state!(self);
    }

    fn check_int(&mut self) {
        if !self.int_polling_enabled {
            return
        }
        if self.reset_trigger || self.nmi_trigger ||
            (!self.regs.int_disabled() && self.irq_trigger) {
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

    /// スタックへのPushと、スタックポインタの減算をまとめて行う。
    pub fn push_stack(&mut self, data: u8) {
        self.set_to_stack(data);
        self.dec_stack();
    }

    /// スタックが新しい空き領域を指すように、スタックポインタを 1 減算する。
    /// スタックへ値を積んだ(Pushした)後に呼び出す。
    pub fn dec_stack(&mut self) {
        // スタックに積みすぎて天井を超えていないか？
        // つまり減算しすぎて $00 -> $FF にオーバーラップしていないか？のチェック。
        check_stack_overflow!(self);
        self.regs.s = self.regs.s.wrapping_sub(1);
    }

    /// 現在のスタックポインタの指すアドレスに値を設定。スタックポインタは操作しない。
    pub fn set_to_stack(&mut self, data: u8) {
        let addr = ADDR_STACK_UPPER | (self.regs.s as u16);
        self.mem.write(addr, data);   
    }

    /// スタックポインタの加算と、Pullをまとめて行う。
    pub fn pull_stack(&mut self) -> u8 {
        self.inc_stack();
        self.peek_stack()
    }

    /// スタックポインタに 1 加算する。スタックからの値の取得(Pull/Pop)の前に呼び出す。
    pub fn inc_stack(&mut self) {
        // スタックから取り出しすぎて底が抜けていないか？
        // つまり加算しすぎて $FF -> $00 にオーバーラップしていないか？のチェック。
        check_stack_underflow!(self);
        self.regs.s = self.regs.s.wrapping_add(1);
    }

    /// 現在のスタックポインタの指すアドレスから値を取り出す。スタックポインタは操作しない。
    pub fn peek_stack(&mut self) -> u8 {
        let addr = ADDR_STACK_UPPER | (self.regs.s as u16);
        let data = self.mem.read(addr);
        data
    }

    #[cfg(debug_assertions)]
    /// スタックが天井を突き破らないかチェック。
    fn check_stack_overflow(&self) {
        if self.regs.s <= 0 {
            log::debug!("!!! stack overflow detected.");
        }
    }

    #[cfg(debug_assertions)]
    /// スタックの底が抜けないかチェック。
    fn check_stack_underflow(&self) {
        if self.regs.s >= u8::MAX {
            log::debug!("!!! stack underflow detected.");
        }
    }

    #[cfg(debug_assertions)]
    fn print_cpu_state(&self) {
        log::debug!("#### CPU STATE: BEGIN");
        log::debug!("PC = {:#X}({})", self.regs.pc, self.regs.pc);
        log::debug!("A = {}, X = {}, Y = {}", self.regs.a, self.regs.x, self.regs.y);
        log::debug!("S = {:#X}({}), P = {:#010b}({})", self.regs.s, self.regs.s, self.regs.p, self.regs.p);
        log::debug!("#### END");
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

