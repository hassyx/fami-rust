//! CPU側の Memory Controller。
//! ミラー領域への値の反映など、メモリへの読み書きを仲介する。

use std::ops::Range;

use crate::nes::ppu;

/// NESに搭載されている物理RAM容量(bytes)
pub const REAL_RAM_SIZE: usize = 0x0800;
/// メモリ空間の広さ(bytes)
pub const RAM_SPACE: usize = 0xFFFF;

pub struct MemCon<'a> {
    ppu: &'a mut ppu::Ppu,
    ram: Box<[u8]>,
}

impl<'a> MemCon<'a> {
    pub fn new(ppu: &'a mut ppu::Ppu) -> Self {
        MemCon {
            ppu,
            ram: Box::new([0; RAM_SPACE]),
        }
    }

    /// ミラーリング等を考慮せず、メモリに直にデータを書き込む。
    /// 主に初期化処理に利用する。
    pub fn raw_write(&mut self, addr: usize, data: &[u8]) {
        println!("mem::MemCon::raw_write() addr={}, data.len()={}", addr, data.len());
        self.ram[addr..addr+data.len()].copy_from_slice(data);
    }

    pub fn raw_write_b(&mut self, addr: usize, data: u8) {
        println!("mem::MemCon::raw_write_b() addr={}", addr);
        self.ram[addr] = data;
    }

    pub fn write(&mut self, addr: usize, data: &[u8]) {
        println!("mem::MemCon::write() addr={}, data.len()={}", addr, data.len());
        self.ram[addr..addr+data.len()].copy_from_slice(data);
    }

    pub fn write_b(&mut self, addr: usize, data: u8) {
        println!("mem::MemCon::write_b() addr={}", addr);
        self.ram[addr] = data;
    }

    pub fn read(&self, range: Range<usize>) -> &[u8] {
        println!("mem::MemCon::read() range={:?}", range);
        &self.ram[range]
    }

    pub fn read_b(&self, addr: usize) -> u8 {
        println!("mem::MemCon::read_b() addr={}", addr);
        self.ram[addr]
    }

    pub fn fill(&mut self, range: Range<usize>, data: u8) {
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