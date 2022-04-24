//! CPU側の Memory Controller。
//! ミラー領域への値の反映など、メモリへの読み書きを仲介する。

use std::cell::RefCell;
use std::rc::Rc;

use std::ops::RangeInclusive;
use num_traits::FromPrimitive;

use crate::util;
use crate::nes::ppu_databus::PpuDataBus;

/// NESに搭載されている物理RAM容量(bytes)
pub const PHYSICAL_RAM_SIZE: usize = 0x0800;
/// 論理メモリ空間(bytes)
pub const LOGICAL_RAM_SPACE: usize = 0x10000;

pub struct MemCon {
    pub ram: Box<[u8]>,
    pub ppu: Rc<RefCell<dyn PpuDataBus>>,
}

impl MemCon {
    
    pub fn new(ppu_databus: Rc<RefCell<dyn PpuDataBus>>) -> Self {
        MemCon {
            ppu: ppu_databus,
            ram: Box::new([0; LOGICAL_RAM_SPACE]),
        }
    }

    /// メモリマップドI/Oやミラー領域を考慮せず、メモリに直にデータを書き込む。
    pub fn raw_write(&mut self, addr: u16, data: &[u8]) {
        log::debug!("raw_write: addr={:#06X}, data.len={}", addr, data.len());
        let addr = addr as usize;
        self.ram[addr..addr+data.len()].copy_from_slice(data);
    }

    /// メモリマップドI/Oやミラー領域を考慮せず、メモリに直にデータを書き込む。
    pub fn raw_write_b(&mut self, addr: u16, data: u8) {
        log::debug!("raw_write_b: addr={:#06X}, data={:#04X}({})", addr, data, data);
        let addr = addr as usize;
        self.ram[addr] = data;
    }

    /// メモリマップドI/Oやミラー領域を考慮せず、メモリに直にデータを書き込む。
    pub fn raw_fill(&mut self, range: RangeInclusive<usize>, data: u8) {
        log::debug!("raw_fill: range=({:?}), data={:#04X}({})", range, data, data);
        self.ram[range].fill(data);
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        log::debug!("write: addr={:#06X}, data={:#04X}({})", addr, data, data);
        match addr {
            // 物理RAM領域への書き込み
            0x0000..=0x1FFF => {
                // 物理RAMのミラー領域への反映
                // orignal:($0000-$07FF) -> mirror:($0800-$0FFF, $1000-$17FF, $1800-$1FFF)
                let addr = addr as usize;
                self.ram[0x0000+addr] = data;
                self.ram[0x0800+addr] = data;
                self.ram[0x1000+addr] = data;
                self.ram[0x1800+addr] = data;
            },
            // PPUへのDMA転送開始アドレスの指定
            0x4014 => {
                // TODO:
                // ここに書かれたアドレスをsrc, PPUのSPR-RAMをdstとしてDMA転送開始。
                // 転送が完了するまでCPUは停止する。つまり新規stateが必要？
                // メモリ上の値もユーザーが指定したデータ(アドレス値)を書き込んでおくこと！
            },
            // PPUのレジスタへの書き込み
            0x2000..=0x3FFF => {
                // ミラー領域への反映を行う
                // 仮にミラー領域へ書きこんでいても、まずはオリジナル領域($2000-$2007)への書き込みとみなす。
                // ここで必要なアドレスは最後の3bitだけ。
                let offset = (addr as usize) & 0x0111;
                let reg_type = FromPrimitive::from_usize(offset).unwrap();
                self.ppu.borrow_mut().write(reg_type, data);

                // 改めて、ミラー領域への反映
                // orignal:($2000-$2007) -> mirror:($2008-$3FFF, repeat evry 8 bytes)
                for i in (0x2008..=0x3FFF).step_by(8) {
                    self.ram[i+offset] = data;
                }
            },
            0x8000..=0xFFFF => {
                // TODO: MapperによってはROMへの書き込みを検出する機構がある。

                // 実機ではROMへの書き込みはエラーとならないが、
                // 現状のエミュレーター実装でROMへの書き込みが行われた場合、
                // 命令デコードの不具合である可能性が高いため、panic させる。
                util::panic_write_to_read_only_area(addr, data)
            },
            // TODO: APUの対応が必要
            _ => {
                // デバイスではなくRAMへ書き込む
                self.ram[addr as usize] = data
            },
        }
    }
    
    pub fn read(&mut self, addr: u16) -> u8 {
        let data = match addr {
            // PPUのレジスタを読む
            0x2000..=0x3FFF => {
                // 仮にミラー領域を読み込んでいても、オリジナル領域($2000-$2007)からの読み込みとみなす。
                // ここで必要なアドレスは最後の3bitだけ。
                let offset = (addr as usize) & 0x0111;
                let reg_type = FromPrimitive::from_usize(offset).unwrap();
                self.ppu.borrow_mut().read(reg_type)
            },
            // TODO: APUの対応が必要
            _ => {
                // デバイスではなくRAMから読み込む
                self.ram[addr as usize]
            },
        };
        log::debug!("read: addr={:#06X}, data={:#04X}({})", addr, data, data);
        data
    }
}
