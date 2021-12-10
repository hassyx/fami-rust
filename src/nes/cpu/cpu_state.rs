//! CPUの状態遷移

use super::{Cpu, Flags, IntType, decoder, executer::Executer};
use crate::util::*;

// 割り込みハンドラのアドレス:
const ADDR_INT_NMI: u16        = 0xFFFA;
const ADDR_INT_RESET: u16      = 0xFFFC;
const ADDR_INT_IRQ: u16        = 0xFFFE;

pub type FnState = fn(&mut Cpu);

/// 一時的な状態保持用
// TODO: 同時に利用しないメンバはunionとして扱った方がいいかも。
pub struct TmpState {
    pub counter: u8,
    pub op_1: u8,
    pub op_2: u8,
    pub addr: u16,
    pub int: IntType,
    pub executer: Executer,
}

impl Default for TmpState {
    fn default() -> Self {
        Self {
            counter: 0,
            op_1: 0,
            op_2: 0,
            addr: 0,
            int: IntType::None,
            executer: Default::default(),
        }
    }
}

impl Cpu {

    //   +--------------------+
    //   |                    ^
    //   v                    |
    // fetch ---+---> exec -->+
    //          |      |      ^
    //          v      v      |
    //          +---> int --->+

    /// OPコードをフェッチする。
    /// Brkだった場合は割り込み状態へ遷移、それ以外は実行状態へ遷移。
    pub fn fetch_step(&mut self) {
        // 命令の実行が完了するまで、割り込み処理のポーリングを止める。
        self.int_polling_enabled = false;

        let opcode = self.fetch();
        if opcode == 0 {
            self.irq_trigger = true;
            self.irq_is_brake = true;
            self.int_1st_clock();
            self.fn_step = Cpu::int_step;
        } else {
            self.state.executer = decoder::decode(self, opcode);
            self.fn_step = Cpu::exec_step;
        }
    }

    /// 命令実行のステップ処理
    pub fn exec_step(&mut self) {
        (self.state.executer.fn_exec)(self);
    }

    /// 割り込みシーケンスのステップ処理。
    /// 割り込み発生を検知、またはフェッチした命令がBrkだった場合にここに来る。
    /// 割り込み種別を判別し、適切なアドレスへジャンプする。
    pub fn int_step(&mut self) {
        match self.state.counter {
            1 => {
                // Brkの場合はすでに1クロック目を通過済みなので、ここには入らない。
                self.int_1st_clock();
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
                // clock 6,7: ジャンプする先の割り込みハンドラのアドレスを読み込む。
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
                // ポーリングが有効になるのは、少なくともこのあと、1つの命令の実行が完了してから。
                self.switch_state_fetch();
            },
            _ => unreachable!(),
        };
    }

    /// 割り込みの第1クロック目に行う処理。
    fn int_1st_clock(&mut self) {
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
    }
}
