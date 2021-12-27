//! PPUのVRAMを管理する Memory Controller。
//! ミラー領域への値の反映など、メモリへの読み書きを仲介する。

/// PPUに搭載されているVRAM容量(bytes)
pub const REAL_VRAM_SIZE: usize = 0x800;
/// メモリ空間の広さ(bytes)
pub const VRAM_SPACE: usize = 0x4000;

/// 16KB(14bit)のメモリ空間を持ち、物理的には2KBの容量を持つVRAMのメモリコントローラー。
/// VRAMに直接アクセスできるのはPPUだけ。CPUは、CPU側のメモリ空間に露出している
/// PPUの2つのレジスタ、PPUADDR($2006)とPPUDATA($2007)を利用してVRAMへアクセスする。
pub struct MemCon {
    ram: Box<[u8]>,
}

/*
VRAM Memory Map:
----------------------  -------------------------
$0000-$0FFF $1000(4KB)  Pattern table 0     (CHR-ROM) 
$1000-$1FFF $1000(4KB)  Pattern table 1     (CHR-ROM)
$2000-$23FF $0400(1KB)  Nametable 0 (左上)  (VRAM or ミラー)
$2400-$27FF $0400(1KB)  Nametable 1 (右上)  (VRAM or ミラー)
$2800-$2BFF $0400(1KB)  Nametable 2 (左下)  (VRAM or ミラー)
$2C00-$2FFF $0400(1KB)  Nametable 3 (右下)  (VRAM or ミラー)
$3000-$3EFF $0F00(4KB)  Mirrors of $2000-$2EFF
$3F00-$3F1F $0020(6bit) Palette RAM indexes
$3F20-$3FFF $00E0       Mirrors of $3F00-$3F1F
----------------------  -------------------------
解説：
・パターンテーブル($0000-$1FFF)は、通常ROM側のCHR-ROMがマッピングされている。
  マッパーによっては内容が動的に切り替わる場合がある。
・ネームテーブル($2000-$2FFF)は、通常2KBのVRAMにマッピングされる。
  水平ミラーリングの場合は $2000-$23FF(左上) と $2800-$2BFF(左下)、
  垂直ミラーリングの場合は $2000-$23FF(左上) と $2400-$27FF(右上) になる。
・$3000-$3EFF は $2000-$2EFF の内容が丸々ミラーリングされている。
・$3F20-$3FFF は $3F00-$3F1F(つまりPalette RAM indexes)が7回ミラーリングされている。
・$3F00-$3F1F は 6bitのパレット領域。
*/

/*
ネームテーブルのビット構成：
ネームテーブルの個々の要素は1バイトで、パターンテーブルへのインデックス(0-255)になっている。
*/

/*
パターンテーブルのビット構成：
パターンテーブルの個々のタイルは16バイトで、サイズは8x8のタイルとなる。
8x8の個々のピクセルが2bitの値(どのパレットを利用するか。パレットへのインデックス)を持つ。
16バイトのデータの内訳としては、最初の8バイトがパレットインデックスの下位1bit、
次の8バイトがパレットインデックスの上位1bitを表す。
*/

/*
属性テーブルのビット構成：
属性テーブルは全体で64バイトで、個々のデータ(タイル4枚分)は1バイト。
1バイトを2bitごとに分けて、全部で8bitで4枚分のタイルのパレットのインデックスを表す。
(パレットの中の4色のどの色を使うか、という情報)
    7654 3210
    |||| ||++- 左上のタイルのパレットインデックス
    |||| ++--- 右上のタイルのパレットインデックス
    ||++------ 左下のタイルのパレットインデックス
    ++-------- 右下のタイルのパレットインデックス
*/

/*
VRAM中のパレット領域($3F00-$3F1F)の詳細：
(要注意) $3F10/$3F14/$3F18/$3F1C は、$3F00/$3F04/$3F08/$3F0C のミラー。
----------- ------------------------
$3F00       Universal background color
$3F01-$3F03 Background palette 0
$3F05-$3F07 Background palette 1
$3F09-$3F0B Background palette 2
$3F0D-$3F0F Background palette 3
$3F11-$3F13 Sprite palette 0
$3F15-$3F17 Sprite palette 1
$3F19-$3F1B Sprite palette 2
$3F1D-$3F1F Sprite palette 3
----------- ------------------------
解説：
・大まかに分けると$3F00-$3F0FはBG用、$3F10-$3F1Fはスプライト用。
・$3F00/$3F04/$3F08/$3F0C使われない領域。
・Universal background color($3F00)は、NESの描画領域外を塗りつぶす色。
  具体的には、オーバースキャン領域に適用される背景色。
・Universal background color以外の、全てのパレットは3色を持つ。
・BGは属性テーブルによって2x2のブロックに1つのパレットが割り当てられる。
・1つのスプライトにつき、PPUOAMのスプライト情報でパレットが1つ指定される。
*/

/*
パレットのビット構成：
パレットの1色には1バイトが割り当てられているが、実際に使われるのは6bitのみ。

76543210
||||||||
||||++++- Hue (phase, determines NTSC/PAL chroma)
||++----- Value (voltage, determines NTSC/PAL luma)
++------- Unimplemented, reads back as 0
*/

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