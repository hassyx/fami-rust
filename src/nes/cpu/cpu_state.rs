//! CPUの状態遷移

use super::{Cpu, Flags, IntType, decoder, executer::Executer};
use crate::util::*;

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
    pub executer: Executer,
}

impl Default for TmpState {
    fn default() -> Self {
        Self {
            counter: 0,
            op_1: 0,
            op_2: 0,
            int: IntType::None,
            executer: Default::default(),
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
            self.state.executer = decoder::fetch_and_decode(self);
        } else {
            // 2クロック目以降は、実際の処理を担う関数に全てを任せる。
            (self.state.executer.fn_exec)(self);
        }
    }

    /// 割り込みシーケンスのステップ処理
    pub fn int_step(&mut self) {
        match self.state.counter {
            1 => {
                // Brkであれば、本来ここで命令をフェッチする必要があるが、
                // 既にフェッチしてBrkかどうかを判定済みの状態でここに来るはずなので、
                // フェッチは必要ない。

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
                        self.state.int = IntType::Brk;
                    } else {
                        self.state.int = IntType::Irq;
                    }
                }
                // TODO: 本来は割り込み種別ごとにトリガーが解除されるタイミングが異なる。
                // また、割り込みが競合した際の振る舞いも実装する必要がある。
                // ここでは、ひとまず一括で現在の割り込み状態をリセットする。
                self.clear_all_int_trigger();
            },
            2 => {
                if self.state.int == IntType::Brk {
                    // Brkの場合、ここに来た時点でPCはBrkの1バイト先を指しているので、更に+1する。
                    self.regs.pc = self.regs.pc.wrapping_add(1);
                }
            },
            // Resetの場合はスタックを操作しない
            3 => if self.state.int != IntType::Reset {
                self.push_stack((self.regs.pc >> 8 & 0x00FF) as u8);
            },
            // Resetの場合はスタックを操作しない
            4 => if self.state.int != IntType::Reset {
                self.push_stack((self.regs.pc & 0x00FF) as u8);
            },
            // Resetの場合はスタックを操作しない
            5 => if self.state.int != IntType::Reset {
                // ステータスレジスタをスタックに保存。
                // その前にBrakeフラグを設定する。Brakeフラグはスタック上にのみ存在する。
                let brk_flag = ((self.state.int == IntType::Brk) as u8) << 4;
                let flags = self.regs.p | brk_flag;
                self.push_stack(flags);
            },
            6 => (),
            7 => {
                // clock 6,7: 割り込みベクタテーブルを読み込む。
                // 処理が重いのでこのクロック内でまとめて処理する。
                let vec_addr = match self.state.int {
                    IntType::Reset => ADDR_INT_RESET,
                    IntType::Nmi => ADDR_INT_NMI,
                    IntType::Irq | IntType::Brk => ADDR_INT_IRQ,
                    IntType::None => unreachable!(),
                };
                let low = self.mem.read(vec_addr);
                let high = self.mem.read(vec_addr+1);
                // clock 8: 割り込みベクタを実行する(ここでは準備だけ)
                self.regs.pc = make_addr(high, low);
                // この時点ではまだ割り込み検出のポーリング処理は停止している。
                // ポーリングが有効になるのは、少なくとも1つの命令の実行が完了してから。
                self.switch_state_exec();
            },
            _ => unreachable!(),
        };
    }
}
