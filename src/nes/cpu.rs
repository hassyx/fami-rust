//! 6502 emulator.

#[macro_use] mod macros;
mod cpu_state;
mod decoder;
mod executer;
mod exec_core_g1;
mod exec_core_g2;
mod exec_core_g3;
mod is_template;
mod is_core;
mod instruction;

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
    // clock_freq: u32,
    // clock_cycle: f32,
    /// 起動後、リセットまたは電源断まで増加し続けるカウンター
    clock_counter: u64,
    regs: Registers,
    /// RESETが発生していたらtrue。物理的なPINはレベルセンシティブ。
    reset_occurred: bool,
    /// NMIが発生していたらtrue。物理的なPINはエッジセンシティブ。
    nmi_occurred: bool,
    /// IRQが発生していたらtrue。物理的なPINはレベルセンシティブ。
    irq_occurred: bool,
    /// 割り込みピンの状態をポーリング可能かどうか。割り込み処理中(ハンドラに遷移する前)にはfalseになる。
    int_polling_enabled: bool,
    /// CPUの状態ごとに切り替わる関数。いわゆるStateパターンを実現するための仕組み。
    /// こうした理由は、1クロックサイクルごとに走る条件判定処理をできるだけ減らしたかったのと、
    /// CPUのメインループ内で呼ばれる処理では、可能な限り動的なメモリ確保を避けたいため、
    /// 構造体ではなく関数ポインタで実現している。(動的な状態はCpu構造体の方に持たせている)
    fn_step: FnState,
    /// 現在実行中の命令が完了したら、次に処理されるべき割り込み。
    int_requested: Interrupt,
    /// 1つの状態が終わるまでの間、必要な情報を一時的に保持する。
    state: TmpState,
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
    /// Status Flag:
    /// 個々のフラグのON/OFFや、分岐命令を通したフラグの状態確認は可能だが、
    /// ユーザー側がこのレジスタ「全体」を直接読み取る命令は存在しない。
    /// PHPか、または割り込み処理の過程によって、メモリ上(スタック上)に
    /// 積まれたレジスタの内容を読み取る必要がある。
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

    /// val1 + val2 + carry
    fn add_with_carry(val1: u8, val2: u8, carry: bool) -> (u8, bool) {
        // 全ての値をu8から16bitに拡張した上で加算を行う。
        // u8が取りうる最大の値が、Carry含めて加算された場合の結果は、以下の通り。
        //
        // $FF + $FF = $1FE(=0b0000_0001_1111_1110)
        // $1FE + 1  = $1FF(=0b0000_0001_1111_1111)
        //
        // つまりキャリー含めて全て加算しても結果は最大 $1FF となり、
        // これらを加算する順番を考慮する必要はなく、合計して得られた値の
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

    /// val1とval2の比較
    fn cmp(&mut self, val1: u8, val2: u8) {
        // 比較処理の実際は、val1 に対して val2 の2の補数を加算し、
        // 加算の際にCarryを考慮せず、計算後にOverflowが変化しないADC。

        let (result, carry) = 
            Self::add_with_carry(val1, val2.wrapping_neg(), false);

        // 桁溢れが発生していたらCarryをOn。そうでなければクリア。
        self.p = (self.p & !Flags::CARRY.bits) | carry as u8;
        // 演算結果のMSBが 1 なら、ZeroをOn。そうでなければクリア。
        self.change_negative_by_value(result);
        // 演算結果が 0 なら、ZeroをOn。そうでなければクリア。
        self.change_zero_by_value(result);
    }

    /// レジスタAに値を設定し、同時にNegativeとZeroフラグを更新する。
    pub fn a_set(&mut self, val: u8) {
        self.a = val;
        self.change_negative_by_value(val);
        self.change_zero_by_value(val);
    }

    /// レジスタXに値を設定し、同時にNegativeとZeroフラグを更新する。
    pub fn x_set(&mut self, val: u8) {
        self.x = val;
        self.change_negative_by_value(val);
        self.change_zero_by_value(val);
    }

    /// レジスタYに値を設定し、同時にNegativeとZeroフラグを更新する。
    pub fn y_set(&mut self, val: u8) {
        self.x = val;
        self.change_negative_by_value(val);
        self.change_zero_by_value(val);
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

pub struct Interrupt {
    kind: IntType,
    /// 現在の命令の完了時ではなく、次の命令の完了時まで発生が遅延されている割り込みの場合はtrue。
    /// 6502のバグを再現するために必要。(具体的には、分岐命令でジャンプが発生し、ジャンプ先の
    /// アドレスがページ内だった場合に、割り込みの発生が1命令遅延される挙動を再現するため。)
    is_force_delayed: bool,
}

impl Default for Interrupt {
    fn default() -> Self {
        Self {
            kind: IntType::None,
            is_force_delayed: false,
        }
    }
}

impl Cpu {
    pub fn new(rom: &Box<rom::NesRom>, ram: Box<mem::MemCon>) -> Self {

        let mut my = Cpu {
            mem: ram,
            // clock_freq: CLOCK_FREQ_NTSC, // Use NTSC as default.
            // clock_cycle: 1f32 / (CLOCK_FREQ_NTSC as f32),
            clock_counter: 0,
            reset_occurred: false,
            nmi_occurred: false,
            irq_occurred: false,
            int_polling_enabled: false,
            regs: Registers::default(),
            fn_step: Cpu::int_step,
            int_requested: Default::default(),
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
        // 電源ON時のCPU状態
        // https://wiki.nesdev.org/w/index.php/CPU_power_up_state

        // レジスタとメモリの初期化
        self.regs.a = 0;
        self.regs.x = 0;
        self.regs.y = 0;
        self.regs.s = 0xFD;
        //self.regs.p = 0x34;
        self.regs.flags_on(Flags::INT_DISABLE | Flags::BREAK | Flags::RESERVED);

        // 物理RAMの初期化。
        // 機種によっては起動時のメモリ内容が一定でない場合もあるが、
        // ここでは0クリアとしておく。
        self.mem.raw_fill(0x0000..=0x07FF, 0);
        
        // APU状態のリセット…だが、既にメモリが0クリアされているので不要。
        /*
        self.mem.raw_fill(0x4000..=0x400F, 0);
        self.mem.raw_fill(0x4010..=0x4013, 0);
        self.mem.write(0x4015, 0);
        self.mem.write(0x4017, 0);
        */

        // 割り込み状態の初期化
        self.clear_all_int_trigger();

        // 電源投入時はReset割り込みから実行開始 
        self.int_requested.kind = IntType::Reset;
        self.int_requested.is_force_delayed = false;
        self.switch_state_int();
    }

    /// 1クロックサイクル進める。
    pub fn step(&mut self){
        self.clock_counter += 1;
        self.state.counter += 1;
        (self.fn_step)(self);

        // 最後の1クロック目の直前にのみ、例外のチェックを行う。
        if self.int_polling_enabled &&
            (self.int_requested.kind == IntType::None) &&
            ((self.state.executer.last_cycle - self.state.counter) == 1)
        {
            self.check_int();
        }

        print_cpu_state!(self);
    }

    /// NMIの発生をCPUに通知。実機での「ピンをhighからlowへ」に相当。
    /// NMIは投げっぱなしで問題ないので、外部から明示的にOFFにする必要はない。
    /// (割り込みハンドラ遷移前にCPU側でOFFにするので)
    pub fn trigger_nmi(&mut self) {
        self.nmi_occurred = true;
    }

    /// IRQの発生をCPUに通知。実機での「ピンをhighからlowへ」に相当。
    pub fn trigger_irq(&mut self) {
        self.irq_occurred = true;
    }

    /// IRQの原因となった事象が解消されたことをCPUに通知。
    /// 実機での「ピンをlowからhighへ」に相当。
    pub fn stop_irq(&mut self) {
        self.irq_occurred = false;
    }

    /// RESETの発生をCPUに通知。実機での「ピンをhighからlowへ」に相当。
    /// IRQと同じく、本来は接続されたデバイス(リセットボタン？)によって、
    /// 信号をhighに戻す必要があるのだが、そこまで厳密にエミュレートはしない。
    /// NMIと同様に、割り込みハンドラ遷移前にCPU側が勝手にOFFにする実装としておく。
    pub fn trigger_reset(&mut self) {
        self.reset_occurred = true;
    }

    /// 例外のポーリング動作
    fn check_int(&mut self) {
        if self.reset_occurred || self.nmi_occurred ||
            (!self.regs.int_disabled() && self.irq_occurred) {
            // 割り込みが発生しているなら、ひとまずその状態を記憶。
            // ここに来た時点でまだ命令の実行中なので、命令終了時に割り込み処理に移る。
            self.int_requested.kind = self.resolve_int_type();
            self.int_requested.is_force_delayed = false;
        }
    }

    fn clear_all_int_trigger(&mut self) {
        self.reset_occurred = false;
        self.nmi_occurred = false;
        self.irq_occurred = false;
    }

    /// PCが指すメモリを1バイト読み、PCを1進める。
    pub fn fetch(&mut self) -> u8 {
        let data = self.mem.read(self.regs.pc);
        self.regs.pc = self.regs.pc.wrapping_add(1);
        data
    }

    fn switch_state_fetch(&mut self) {
        // 次の命令をフェッチする前に、予約されている割り込みがあればそちらを先に処理。
        if self.int_requested.kind != IntType::None {
            // 割り込みの発生を1命令遅延するように指定されているか？
            if self.int_requested.is_force_delayed {
                // 割り込みは「次の命令」の直前に処理するので、今回はフラグを落とすだけで何もしない。
                self.int_requested.is_force_delayed = false;
            } else {
                self.switch_state_int();
                return;
            }
        }

        // 割り込みを処理しない場合は、命令のフェッチ処理へ遷移。
        self.state = TmpState::default();
        self.fn_step = Cpu::fetch_step;
    }

    fn switch_state_int(&mut self) {
        self.state = TmpState::default();
        self.state.int = self.int_requested.kind;
        self.int_requested = Default::default();
        self.fn_step = Cpu::int_step;
        self.int_polling_enabled = false;
    }

    /*
    fn switch_state_exec(&mut self) {
        self.state = TmpState::default();
        self.fn_step = Cpu::exec_step;
    }
    */

    fn exec_finished(&mut self) {
        self.switch_state_fetch();
    }

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

    /// 割り込みピンの状態を調べ、どの割り込みを発生させるかを決定する。
    /// 同時に、必要であればピンの状態を変更する。
    fn resolve_int_type(&mut self) -> IntType {
        // 発生した割り込み種別をチェックして記憶
        // 優先度: Reset > NMI > IRQ = Brk
        if self.reset_occurred {
            // RESETはリセットボタンの上げ下げによってPINの状態が変化するが、
            // エミュレーター実装としてはここで離した(lowからhighになった)ものとする。
            self.reset_occurred = false;
            return IntType::Reset
        } else if self.nmi_occurred {
            // NMIの発生状況はフリップフロップに記録されているので、ここで消去。
            self.nmi_occurred = false;
            return IntType::Nmi
        } else if self.irq_occurred {
            // BRKは命令フェッチ時に処理しているので、ここには来ない。
            return IntType::Irq
            // IRQは発生元のデバイスがピンを明示的にhighに戻す必要がある。
            // なのでここではピンを操作しない。
        }
        // 割り込みの発生を前提としてこの関数を呼ぶので、ここに来たらバグ。
        unreachable!()
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
        log::debug!("PC = {:#06X}({})", self.regs.pc, self.regs.pc);
        log::debug!("A = {}, X = {}, Y = {}", self.regs.a, self.regs.x, self.regs.y);
        log::debug!("S = {:#04X}({}), P = {:#010b}({})", self.regs.s, self.regs.s, self.regs.p, self.regs.p);
        log::debug!("#### END");
    }
}

