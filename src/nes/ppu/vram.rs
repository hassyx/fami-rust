//! PPUのVRAMを管理する Memory Controller。
//! ミラー領域への値の反映など、メモリへの読み書きを仲介する。

/// PPUに搭載されているVRAM容量(bytes)
pub const REAL_VRAM_SIZE: usize = 0x800;
/// メモリ空間の広さ(bytes)
pub const VRAM_SPACE: usize = 0x4000;

/// 64KBのメモリ空間を持ち、物理的には16KBの容量を持つVRAMのメモリコントローラー。
pub struct MemCon {
    ram: Box<[u8]>,
}

impl MemCon {
    pub fn new() -> Self {
        Self {
            ram: Box::new([0; VRAM_SPACE]),
        }
    }

    /// ミラーリング等を考慮せず、メモリに直にデータを書き込む。
    /// 主に初期化処理に利用する。
    pub fn raw_write(&mut self, addr: usize, data: &[u8]) {
        println!("addr={}, data.len()={}", addr, data.len());
        self.ram[addr..addr+data.len()].copy_from_slice(data);
    }
}