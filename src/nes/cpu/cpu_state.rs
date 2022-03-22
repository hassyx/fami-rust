//! CPUの状態遷移

use super::{Cpu, Flags, IntType, decoder, executer::Executer};
use crate::util::*;

// 割り込みハンドラのアドレス:
const ADDR_INT_NMI: u16        = 0xFFFA;
const ADDR_INT_RESET: u16      = 0xFFFC;
const ADDR_INT_IRQ: u16        = 0xFFFE;

const OPCODE_BRK: u8 = 0;

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

    //   +-----------------------------------------------+
    //   |                                               ^
    //   v                                               |
    // fetch ---+---(not BRK)---> exec --(exec finish)-->+
    //          |                  |                     ^
    //        (BRK)          (exec finish)               |
    //          v                  v                     |
    //          +---------------> int ------------------>+

    /// OPコードをフェッチする。
    /// Brkだった場合は割り込み状態へ遷移、それ以外は実行状態へ遷移。
    pub fn fetch_step(&mut self) {
        log::debug!("[Fetch] counter={}", self.state.counter);
        // 命令の実行が完了するまで、割り込み処理のポーリングを止める。
        self.int_polling_enabled = false;

        let opcode = self.fetch();
        log::debug!("[Fetch] opcode={:#04X}", opcode);
        if opcode == OPCODE_BRK {
            // まず割り込み状態のポーリングを禁止
            self.int_polling_enabled = false;
            // BRKはソフトウェア割り込みなので、物理的なピンは操作しないし、
            // ピンの状態を上げ下げする必要もない。
            // ここで内部的なフラグを直接立てる。
            self.state.int = IntType::Brk;
            self.fn_step = Cpu::int_step;
        } else {
            self.state.executer = decoder::decode(opcode);
            self.fn_step = Cpu::exec_step;
        }
    }

    /// 命令実行のステップ処理
    pub fn exec_step(&mut self) {
        log::debug!("[Execute] counter={}", self.state.counter);
        (self.state.executer.fn_exec)(self);
    }

    /// 割り込みシーケンスのステップ処理。
    /// 割り込み発生を検知、またはフェッチした命令がBrkだった場合にここに来る。
    /// 割り込み種別を判別し、適切なアドレスへジャンプする。
    pub fn int_step(&mut self) {
        log::debug!("[Interrupt] counter={}", self.state.counter);
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
            6 => {
                // ジャンプする先の割り込みハンドラのアドレス(下位8bit)を読み込む。
                // が、エミュレーター実装としては何もしない(7クロック目でまとめて対応する)。

                // ここでIRQ/BRK無視フラグを立てる
                self.regs.flags_on(Flags::INT_DISABLE);
            },
            7 => {
                // ジャンプする先の割り込みハンドラのアドレス(上位8bit)を読み込む。
                // クロック6で何もしていないので、ここで下位と上位アドレスをまとめて読み込む。
                let vec_addr = match self.state.int {
                    IntType::Reset => ADDR_INT_RESET,
                    IntType::Nmi => ADDR_INT_NMI,
                    IntType::Irq | IntType::Brk => ADDR_INT_IRQ,
                    IntType::None => unreachable!(),
                };
                let low = self.mem.read(vec_addr);
                let high = self.mem.read(vec_addr+1);
                self.regs.pc = make_addr(high, low);
                if self.state.int == IntType::Reset {
                    // リセット時の初期化処理の開始
                    // スタックポインタを3減算(ただしスタックの内容自体は操作しない)
                    self.regs.s = self.regs.s.wrapping_sub(3);
                    // IRQ/BRK無視フラグを立てる
                    self.regs.flags_on(Flags::INT_DISABLE);
                    // TODO: APUの状態リセットが必要
                }
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
        // 発生した割り込み種別をチェックして記憶
        // 優先度: Reset > NMI > IRQ = Brk
        if self.reset_occurred {
            // RESETはリセットボタンの上げ下げによってPINの状態が変化するが、
            // エミュレーター実装としてはここで離した(lowからhighになった)ものとする。
            self.reset_occurred = false;
            self.state.int = IntType::Reset;
        } else if self.nmi_occurred {
            // NMIの発生状況はフリップフロップに記録されているので、ここで諸居。
            self.nmi_occurred = false;
            self.state.int = IntType::Nmi;
        } else if self.irq_occurred {
            // ここはIRQの対応となる。BRKは命令フェッチ時に処理しているので、ここには来ない。
            self.state.int = IntType::Irq;
            // IRQは発生元のデバイスがピンを明示的にhighに戻す必要がある。
            // なのでここではピンを操作しない。
        }
    }
}
