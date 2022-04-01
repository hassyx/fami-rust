//! Instruction decoder.

use super::executer::*;
use super::instruction::*;

fn panic_invalid_op(opcode: u8) -> ! {
    panic!("\"{:#04X}\" is invalid opcode.", opcode);
}

pub fn decode(opcode: u8) -> Executer {
    if let Some(inst) = INSTRUCTION_SET[opcode as usize] {
        return Executer {
            // ひとまず最小の所要クロックを設定しておくが、命令内で変動する可能性がある。
            last_cycle: inst.min_clock,
            inst,
        }
    }
    panic_invalid_op(opcode);
}


