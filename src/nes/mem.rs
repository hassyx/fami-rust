//! CPU側の Memory Controller。
//! ミラー領域への値の反映など、メモリへの読み書きを仲介する。

use std::ops::RangeInclusive;
use num_traits::FromPrimitive;
use range_check::Check;

use crate::nes::ppu_databus::DataBus;

/// NESに搭載されている物理RAM容量(bytes)
pub const PHYSICAL_RAM_SIZE: usize = 0x0800;
/// 論理メモリ空間(bytes)
pub const LOGICAL_RAM_SPACE: usize = 0x10000;

pub struct MemCon {
    ram: Box<[u8]>,
    pub ppu_databus: Box<DataBus>,
}

impl MemCon {
    
    pub fn new(ppu_databus: Box<DataBus>) -> Self {
        MemCon {
            ppu_databus,
            ram: Box::new([0; LOGICAL_RAM_SPACE]),
        }
    }

    /// メモリマップドI/Oやミラー領域を考慮せず、メモリに直にデータを書き込む。
    pub fn raw_write(&mut self, addr: u16, data: &[u8]) {
        println!("mem::MemCon::raw_write() addr={}, data.len()={}", addr, data.len());
        let addr = addr as usize;
        self.ram[addr..addr+data.len()].copy_from_slice(data);
    }

    /// メモリマップドI/Oやミラー領域を考慮せず、メモリに直にデータを書き込む。
    pub fn raw_write_b(&mut self, addr: u16, data: u8) {
        println!("mem::MemCon::raw_write_b() addr={}", addr);
        let addr = addr as usize;
        self.ram[addr] = data;
    }

    /// メモリマップドI/Oやミラー領域を考慮せず、メモリに直にデータを書き込む。
    pub fn raw_fill(&mut self, range: RangeInclusive<usize>, data: u8) {
        println!("mem::MemCon::fill() range={:?}, data={}", range, data);
        self.ram[range].fill(data);
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        println!("mem::MemCon::write_b() addr={}", addr);
        if !self.write_to_dev(addr, data) {
            self.ram[addr as usize] = data;
        }
    }
    
    pub fn read(&mut self, addr: u16) -> u8 {
        println!("mem::MemCon::read_b() addr={}", addr);
        if let Some(data) = self.read_from_dev(addr) {
            data
        } else {
            self.ram[addr as usize]
        }
    }

    // 8bit CPUなので、複数バイトの同時書き込みは不要？
    /*
    pub fn write(&mut self, addr: usize, data: &[u8]) {
        println!("mem::MemCon::write() addr={}, data.len()={}", addr, data.len());
        self.ram[addr..addr+data.len()].copy_from_slice(data);
    }

    pub fn read(&self, range: Range<usize>) -> &[u8] {
        println!("mem::MemCon::read() range={:?}", range);
        &self.ram[range]
    }
    */

    /// メモリ空間上に存在するデバイスへの書き込みや、ミラー領域への反映を(必要であれば)行う。
    /// 書き込みが行われた場合はtrueを返す。
    fn write_to_dev(&mut self, addr: u16, data: u8) -> bool {
        match addr {
            0x0000..=0x1FFF => {
                // 物理RAMのミラー領域への反映
                // orignal:($0000-$07FF) -> mirror:($0800-$0FFF, $1000-$17FF, $1800-$1FFF)
                let addr = addr as usize;
                self.ram[0x0000+addr] = data;
                self.ram[0x0800+addr] = data;
                self.ram[0x1000+addr] = data;
                self.ram[0x1800+addr] = data;
                return true;
            },
            0x2000..=0x3FFF | 0x4014 => {
                // PPUのレジスタへ値を設定し、ミラー領域への反映を行う
                self.write_ppu_register(addr, data);
                return true;
            },
            // TODO: APUの対応が必要
            _ => return false,
        }
    }

    fn read_from_dev(&mut self, addr: u16) -> Option<u8> {
        match addr {
            0x2000..=0x3FFF | 0x4014 => {
                Some(self.read_ppu_register(addr))
            },
            // TODO: APUの対応が必要
            _ => None,
        }
    }

    /// CPUのメモリ空間に露出した、PPUのレジスタへの書き込み
    fn write_ppu_register(&mut self, addr: u16, data: u8) {
        debug_assert_eq!(addr.check_range(0x2000..=0x3FFF | 0x4014), Ok(addr));

        if addr == 0x4014 {
            self.ppu_databus.write_to_oamdma(data);
        } else {
            // 仮にミラー領域へ書きこんでいても、まずはオリジナル領域($2000-$2007)への書き込みとみなす。
            // ここで必要なアドレスは最後の3bitだけ。
            let offset = (addr as usize) & 0x0111;
            let reg_type = FromPrimitive::from_usize(offset & 0x0111).unwrap();
            self.ppu_databus.write(reg_type, data);

            // ミラー領域への反映
            // orignal:($2000-$2007) -> mirror:($2008-$3FFF, repeat evry 8 bytes)
            for i in (0x2008..=0x3FF7).step_by(8) {
                self.ram[i+offset] = data;
            }
        }
    }

    /// CPUのメモリ空間に露出した、PPUのレジスタからの読み込み
    fn read_ppu_register(&mut self, addr: u16) -> u8 {
        debug_assert_eq!(addr.check_range(0x2000..=0x3FFF | 0x4014), Ok(addr));

        if addr == 0x4014 {
            return self.ppu_databus.read_from_oamdma()
        } else {
            // 仮にミラー領域を読み込んでいても、オリジナル領域($2000-$2007)への読み込みとみなす。
            // ここで必要なアドレスは最後の3bitだけ。
            let offset = (addr as usize) & 0x0111;
            let reg_type = FromPrimitive::from_usize(offset & 0x0111).unwrap();
            return self.ppu_databus.read(reg_type)
        }
    }

    pub fn dump(&self) {
        println!("{:?}", self.ram);
    }
}

/*
// 配列のように添字でアクセスできてもいいかなーと思ったが、
// 書き込み前に割り込んで独自処理を入れるのが面倒なので諦めた。
// 読み込み時に関しては Index を使うことで問題なく実装できる。
// (読み込みだけ添字アクセスが有効だと、対称性が失われて嫌なので断念)

use std::ops::Index;
use std::ops::IndexMut;
use std::ops::Range;
impl Index<usize> for MemCon<'_> {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.ram[index]
    }
}
impl<'a> Index<Range<usize>> for MemCon<'a> {
    type Output = [u8];
    fn index(&self, range: Range<usize>) -> &Self::Output {
        &self.ram[range]
    }
}

impl IndexMut<usize> for MemCon<'_> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.ram[index]
    }
}
impl IndexMut<Range<usize>> for MemCon<'_> {
    fn index_mut(&mut self, range: Range<usize>) -> &mut Self::Output {
        &mut self.ram[range]
    }
}
*/