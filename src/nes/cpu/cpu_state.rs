//! CPUの状態遷移

use super::{Cpu, Flags, IntType};
use crate::util::*;
use super::decoder::PtrFnExec;


// 割り込みハンドラのアドレス:
const ADDR_INT_NMI: u16        = 0xFFFA;
const ADDR_INT_RESET: u16      = 0xFFFC;
const ADDR_INT_IRQ: u16        = 0xFFFE;

/// 一時的な状態保持用
pub struct TmpState {
    pub counter: u8,
    pub op_1: u8,
    pub op_2: u8,
    pub int: IntType,
    pub fn_exec: Option<PtrFnExec>,
}

impl Default for TmpState {
    fn default() -> Self {
        Self {
            counter: 0,
            op_1: 0,
            op_2: 0,
            int: IntType::None,
            fn_exec: None,
        }
    }
}

impl Cpu {
    /// 命令実行のステップ処理
    pub fn exec_step(&mut self) {
        // 効率のいい命令デコードについてはここが詳しい。
        // https://llx.com/Neil/a2/opcodes.html

        if self.state.counter == 1 {
            // 命令の実行が完了するまで、割り込み処理のポーリングを止める。
            self.int_polling_enabled = false;
            // 1クロックサイクル目は必ずOPコードのフェッチになる。
            // 命令種別を解析し、実際の処理を担う関数を取得して設定。
            self.state.fn_exec = Some(self.decode());
        } else {
            // 2クロック目以降は、実際の処理を担う関数に全てを任せる。
            self.state.fn_exec.unwrap()(self);
        }
    }

    /// 割り込みシーケンスのステップ処理
    pub fn int_step(&mut self) {
        //　割り込み処理は要7クロック。8クロック目に割り込みベクタの遷移先の実行開始。
        if self.state.counter == 1 {
            // まず割り込み状態のポーリングを禁止
            self.int_polling_enabled = false;
            // IRQ/BRK無視フラグを立てる
            self.regs.flags_on(Flags::INT_DISABLE);
            // 発生した割り込み種別をチェックして記憶
            // 優先度: Reset > NMI > IRQ = Brk
            if self.reset_trigger {
                self.state.int = IntType::Reset;
            } else if self.nmi_trigger {
                self.state.int = IntType::Nmi;
            } else if self.irq_trigger {
                if self.irq_is_brake {
                    self.state.int = IntType::Irq;
                } else {
                    self.state.int = IntType::Brk;
                }
            }
            // TODO: 本来は割り込み種別ごとにトリガーが解除されるタイミングが異なる。
            // また、割り込みが競合した際の振る舞いも実装する必要がある。
            // ここでは、ひとまず一括で現在の割り込み状態をリセットする。
            self.clear_all_int_trigger();
        } else if self.state.counter == 7 {
            // Brkフラグの設定
            if self.state.int == IntType::Brk {
                self.regs.flags_on(Flags::BREAK);
            } else {
                self.regs.flags_off(Flags::BREAK);
            }
            // Resetの場合はスタックを触らない
            if self.state.int != IntType::Reset {
                // clock 3,4: RTIで割り込み処理終了時に戻るアドレス(PC)を、High, Lowの順にpush。
                if self.state.int == IntType::Brk {
                    // Brkの場合、ここに来た時点でPCはBrkの2バイト目を指しているので、更に+1する。
                    self.regs.pc += 1;
                }
                self.push((self.regs.pc >> 8 & 0x00FF) as u8);
                self.push((self.regs.pc & 0x00FF) as u8);
                // clock 5: ステータスレジスタをpush
                self.push(self.regs.p);
            }
            // スタックに保存したあとは、無条件でBreakフラグを落とす(常に0)
            self.regs.flags_off(Flags::BREAK);
            // clock 6,7: 割り込みベクタテーブルを読み込む。
            let vec_addr = match self.state.int {
                IntType::Reset => ADDR_INT_RESET,
                IntType::Nmi => ADDR_INT_NMI,
                IntType::Irq | IntType::Brk => ADDR_INT_IRQ,
                IntType::None => panic!("invalid IntType."),
            };
            let low = self.mem.read(vec_addr);
            let high = self.mem.read(vec_addr+1);
            // clock 8: 割り込みベクタを実行する(ここでは準備だけ)
            self.regs.pc = make_addr(high, low);
            println!("{}", self.regs.pc);
            // この時点ではまだ割り込み検出のポーリング処理は停止している。
            // ポーリングが有効になるのは、少なくとも1つの命令の実行が完了してから。
            self.switch_state_exec();
        }
    }
}


/*
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
*/

/*
//////////////////////////////////////////////////

/// CPUの状態
pub trait CpuState {
    /// 1クロック分進める
    fn step(&mut self, cpu: &mut Cpu);
    /// 内部状態をリセットする。
    fn reset(&mut self);
}

//////////////////////////////////////////////////
*/

/*
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
*/

//////////////////////////////////////////////////

/*
/// 命令実行状態
#[derive(Default)]
pub struct StateExec {

}

impl CpuState for StateExec {
    fn step(&mut self, cpu: &mut Cpu) {
        // 命令解析その他を実行…
    }

    fn reset(&mut self) {
        
    }
}
*/

//////////////////////////////////////////////////

/*
/// 割り込み処理状態
#[derive(Default)]
pub struct StateInt {

}

impl CpuState for StateInt {
    fn step(&mut self, cpu: &mut Cpu) {
        // 命令解析その他を実行…
    }

    fn reset(&mut self) {
        
    }
}
*/

/*
/// 命令実行
fn STATE_exec(cpu: &mut Cpu) {
    // 効率のいい命令デコードについてはここが詳しい。
    // https://llx.com/Neil/a2/opcodes.html

    if cpu.tmp_counter == 1 {
        // 命令の実行が完了するまで、割り込み処理のポーリングを止める。
        cpu.int_polling_enabled = false;
        cpu.execute();
    } else if cpu.exec_finished() {
        cpu.switch_state(STATE_exec);
    }
}

/// 割り込み処理
fn STATE_interrupt(cpu: &mut Cpu) {
    debug_assert!(cpu.tmp_counter < 8);

    //　割り込み処理は要7クロック。8クロック目に割り込みベクタの実行開始。
    
    if cpu.tmp_counter == 1 {
        // まず割り込み状態のポーリングを禁止
        cpu.int_polling_enabled = false;
        // どのタイプの割り込みが発生したのかチェック
        // 優先度: Reset > NMI > IRQ = Brk
        if cpu.reset_trigger {
            cpu.int_type = IntType::Reset;
        } else if cpu.nmi_trigger {
            cpu.int_type = IntType::Nmi;
        } else if cpu.irq_trigger {
            if cpu.irq_is_brake {
                cpu.int_type = IntType::Irq;
            } else {
                cpu.int_type = IntType::Brk;
            }
        }
        // TODO: 本来は割り込み種別ごとにトリガーが解除されるタイミングが異なる。
        // とりあえず一括で状態をクリアする。
        cpu.clear_all_int_trigger();
    } else if cpu.tmp_counter == 7 {
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
        let low = cpu.mem.read(vec_addr);
        let high = cpu.mem.read(vec_addr+1);
        // clock 8: 割り込みベクタを実行する(ここでは準備だけ)
        cpu.regs.pc = ((high as u16) << 4) | low as u16;
        cpu.int_type = IntType::None;
        // この時点ではまだ割り込み検出のポーリング処理は停止している。
        // ポーリングが有効になるのは、少なくとも次の命令が完了してから。
        cpu.switch_state(STATE_exec);
        return;
    }
}
*/