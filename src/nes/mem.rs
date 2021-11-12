//! CPU側の Memory Controller。
//! ミラー領域への値の反映など、メモリへの読み書きを仲介する。

use std::cell::RefCell;
use std::rc::Rc;
use std::ops::RangeInclusive;

use crate::nes::ppu::Ppu;
use crate::nes::ppu_databus::DataBus;
use crate::nes::ppu_databus::PpuRegs;

/// NESに搭載されている物理RAM容量(bytes)
pub const REAL_RAM_SIZE: usize = 0x0800;
/// メモリ空間の広さ(bytes)
pub const RAM_SPACE: usize = 0xFFFF;

pub struct MemCon {
    ram: Box<[u8]>,
    pub ppu_databus: Box<DataBus>,
}

impl MemCon {
    
    pub fn new(ppu: Rc<RefCell<Ppu>>) -> Self {
        MemCon {
            ppu_databus: Box::new(DataBus::new(ppu)),
            ram: Box::new([0; RAM_SPACE]),
        }
    }

    /// メモリ空間上に存在するデバイスへの書き込みや、
    /// ミラー領域への反映を(必要であれば)行う。
    fn write_mapped_dev(&mut self, addr: usize, data: u8) {
        match addr {
            0x0000..=0x07FF => {
                // 物理RAMのミラー領域への反映
                // orignal:($0000-$07FF) -> mirror:($0800-$0FFF, $1000-$17FF, $1800-$1FFF)
                self.ram[0x0800+addr] = data;
                self.ram[0x1000+addr] = data;
                self.ram[0x1800+addr] = data;
            },
            0x2000..=0x2007 | 0x4014 => {
                // PPUのレジスタへ値の設定、かつミラー領域への反映
                // orignal:($2000-$2007) -> mirror:($2008-$3FFF, repeat evry 8 bytes)
                self.write_ppu_register(addr, data);
            },
            // TODO: fixme
            _ => (),
        }
    }

    /// メモリ書き込み時のPPUへのレジスタへの反映
    fn write_ppu_register(
        &mut self,
        addr: usize,
        data: u8)
    {
        // PPUのレジスタへの値の設定、かつミラー領域への反映
        // orignal:($2000-$2007) -> mirror:($2008-$3FFF, repeat evry 8 bytes)
        
        self.ppu_databus.write(PpuRegs::Status, data);

        /*
        match addr {
            0x2000 => self.ppu.regs.ctrl = data,
            0x2001 => self.ppu.regs.mask = data,
            0x2002 => {
                // PPUSTATUS は書き込み禁止。書き込みんだデータはデータバスを埋める。
                
            },
            0x2003 => self.ppu.regs.oam_addr = data,
            0x2004 => self.ppu.regs.oam_data = data,
            0x2005 => self.ppu.regs.scroll = data,
            0x2006 => self.ppu.regs.addr = data,
            0x2007 => self.ppu.regs.data = data,
            0x4014 => self.ppu.regs.oam_dma = data,
            _ => (),
        };

        // ミラー領域への反映
        let offset = addr - 0x2000;
        for i in (0x2008..=0x3FF7).step_by(8) {
            self.ram[i+offset] = data;
        }
        */
    }

    /// メモリマップドI/Oやミラー領域を考慮せず、メモリに直にデータを書き込む。
    pub fn raw_write(&mut self, addr: usize, data: &[u8]) {
        println!("mem::MemCon::raw_write() addr={}, data.len()={}", addr, data.len());
        self.ram[addr..addr+data.len()].copy_from_slice(data);
    }

    /// メモリマップドI/Oやミラー領域を考慮せず、メモリに直にデータを書き込む。
    pub fn raw_write_b(&mut self, addr: usize, data: u8) {
        println!("mem::MemCon::raw_write_b() addr={}", addr);
        self.ram[addr] = data;
    }

    pub fn write(&mut self, addr: usize, data: u8) {
        println!("mem::MemCon::write_b() addr={}", addr);
        self.ram[addr] = data;
        self.write_mapped_dev(addr, data);
    }

    pub fn read(&self, addr: usize) -> u8 {
        println!("mem::MemCon::read_b() addr={}", addr);
        self.ram[addr]
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

    pub fn fill(&mut self, range: RangeInclusive<usize>, data: u8) {
        println!("mem::MemCon::fill() range={:?}, data={}", range, data);
        self.ram[range].fill(data);
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