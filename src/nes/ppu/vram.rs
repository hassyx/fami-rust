//! PPUのVRAMを管理する Memory Controller。
//! ミラー領域への値の反映など、メモリへの読み書きを仲介する。

use crate::util;
use crate::nes::rom::MirroringType;

/// PPUに搭載されているVRAM容量(bytes)
const REAL_VRAM_SIZE: usize = 0x800;
/// メモリ空間の広さ(bytes)
const VRAM_SPACE: usize = 0x4000;

const NAMETABLE_BASE_ADDR: u16 = 0x2000;
const NAMETABLE_MIRROR_BASE_ADDR: u16 = 0x3000;

const NAMETABLE_HORIZONTAL_OFFSET: u16 = 0x800;
const NAMETABLE_VERTICAL_OFFSET: u16 = 0x400;


/// 16KB(14bit)のメモリ空間を持ち、物理的には2KBの容量を持つVRAMのメモリコントローラー。
/// VRAMに直接アクセスできるのはPPUだけ。CPU側からPPUにアクセスするには、CPU側のメモリ空間に
/// 露出しているPPUの2つのレジスタ、PPUADDR($2006)とPPUDATA($2007)を利用する。
pub struct MemCon {
    vram: Box<[u8]>,
    mirroring_type: MirroringType,
}

/*
PPUの物理的なメモリ構成:
・ネームテーブル用のRAM     (2KB)
・スプライト(OAM)用のRAM    (256バイト)
・パレット用のRAM           (不明。ユーザーから見えるのは32バイト)
*/

/*
VRAM Memory Map:
----------------------  ----------------------- --------------------
アドレス                用途                    物理的な位置
----------------------  ----------------------- --------------------
$0000-$0FFF $1000(4KB)  Pattern table 0         (CHR-ROM) 
$1000-$1FFF $1000(4KB)  Pattern table 1         (CHR-ROM)
$2000-$23FF $0400(1KB)  Nametable 0 (左上)      (専用RAM or ミラー)
$2400-$27FF $0400(1KB)  Nametable 1 (右上)      (専用RAM or ミラー)
$2800-$2BFF $0400(1KB)  Nametable 2 (左下)      (専用RAM or ミラー)
$2C00-$2FFF $0400(1KB)  Nametable 3 (右下)      (専用RAM or ミラー)
$3000-$3EFF $0F00(4KB)  Mirrors of $2000-$2EFF
$3F00-$3F1F $0020(32B)  Palette RAM indexes     (専用RAM)
$3F20-$3FFF $00E0(224B) Mirrors of $3F00-$3F1F
----------------------  --------------------------------------------
解説：
・パターンテーブル($0000-$1FFF)は、通常ROM側のCHR-ROMがマッピングされている。
  カートリッジがパターンテーブルを1枚のみ持っている場合は、1枚目が2枚目にミラーリングされる。
  マッパーによってはパターンテーブルの内容が動的に切り替わる場合がある。
・ネームテーブル($2000-$2FFF)は2枚あり、通常2KBのRAM(物理)が利用される。
  全体としては4枚あるのであと2KB足りないが、残り2枚は物理RAMのミラー領域となる。
  水平ミラーリングの場合は $2000(左上/物理) = $2800(左下/ミラー), $2400(右上/物理) = $2C00(右下/ミラー) となる。
  垂直ミラーリングの場合は $2000(左上/物理) = $2400(右上/ミラー), $2800(左下/物理) = $2C00(右下/ミラー) となる。
・ネームテーブルのミラー領域は $3000-$3EFF の3840バイトで、オリジナルの4KBが
  丸ごとミラーリングされているわけではない。(具体的には$100=256バイト足りない)
・$3000-$3EFF は $2000-$2EFF の内容が丸々ミラーリングされている。
・$3F20-$3FFF は $3F00-$3F1F(つまりPalette RAM indexes)が7回ミラーリングされている。
・$3F00-$3F1F は 6bitのパレット領域。
*/

/*
ネームテーブルと4つの画面(ピクセル)の対応表:

     (0,0)     (256,0)     (511,0)
       +-----------+-----------+
       |           |           |
       |           |           |
       |   $2000   |   $2400   |
       |           |           |
       |           |           |
(0,240)+-----------+-----------+(511,240)
       |           |           |
       |           |           |
       |   $2800   |   $2C00   |
       |           |           |
       |           |           |
       +-----------+-----------+
     (0,479)   (256,479)   (511,479)
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
・1バイトで1色を表す。
・$3F04/$3F08/$3F0C はPPUが利用しない領域。
  ただしCPU側から普通に読み書きはできる。
・$3F10/$3F14/$3F18/$3F1C は $3F00/$3F04/$3F08/$3F0C のミラー。
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
    pub fn new(mirroring_type: MirroringType) -> Self {
        Self {
            vram: Box::new([0; VRAM_SPACE]),
            mirroring_type,
        }
    }

    /// ミラーリング等を考慮せず、メモリに直にデータを書き込む。
    /// 主に初期化処理に利用する。
    pub fn raw_write(&mut self, addr: u16, data: &[u8]) {
        log::debug!("raw_write: addr={:#06X}, data.len={}", addr, data.len());
        debug_assert!(addr < 0x3FFF);
        let addr = addr as usize;
        self.vram[addr..addr+data.len()].copy_from_slice(data);
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        log::debug!("write: addr={:#06X}, data={:#04X}({})", addr, data, data);
        debug_assert!(addr < 0x3FFF);
        
        match addr {
            0x0000..=0x1FFF => {
                // パターンテーブル(CHR-ROM)への書き込みはひとまずエラーとしておく。
                // TODO: CHR-"RAM" の場合は書き込みに対応する必要あり。
                util::panic_write_to_read_only_area(addr, data)
            },
            0x2000..=0x3EFF => {
                // ネームテーブル(またはそこへのミラー領域)への書き込み。
                
                // 「ネームテーブル自体のミラーリング(2枚が4枚として扱われる仕組み)」と、
                // 「4枚のネームテーブルがVRAM上にもう1つ存在する」という意味のミラーリングの
                //  2つの仕組みがあってややこしい。
                // 前者は「垂直ミラーリング」または「水平ミラーリング」と表記する。

                // テーブル中の「どの位置への書き込み」かを取得。
                // ここで必要なのはアドレスの下位12bit。
                let pos = addr & 0x0FFF;
                // 水平 or 垂直ミラーの書き込みを実現するため、
                // 指定されたアドレスに加算するオフセット値。
                let offset = match self.mirroring_type {
                    MirroringType::Horizontal => NAMETABLE_HORIZONTAL_OFFSET,
                    MirroringType::Vertical => NAMETABLE_VERTICAL_OFFSET,
                    _ => unreachable!()
                };

                // ここから書き込み。
                // 指定されたアドレスに書き込んだあと、水平ミラーリングの場合は 0x800 を加算、
                // 垂直ミラーリングの場合は 0x400 を加算し、溢れたビットは無視すれば、
                // オリジナル領域と垂直 or 水平ミラー領域の両方へ書き込みが可能。
                // ただし、$3F00-$3F1F がパレット用に利用されている(つまり、全体がミラーされていない)
                // ことに注意が必要。
                
                // 水平 or 垂直にミラーされている領域の、どちらかへの書き込み
                let addr = NAMETABLE_BASE_ADDR | pos;
                self.vram[addr as usize] = data;
                // 水平 or 垂直にミラーされている領域の、さっきとは違う側への書き込み
                let addr = NAMETABLE_BASE_ADDR | pos.wrapping_add(offset);
                self.vram[addr as usize] = data;
                // ここからVRAM上のミラー領域への書き込み
                if pos > 0xEFF {
                    // ミラーのオリジナル領域にだけ書き込む
                    let addr = NAMETABLE_MIRROR_BASE_ADDR | pos.wrapping_add(offset);
                    self.vram[addr as usize] = data;
                } else {
                    // 水平 or 垂直にミラーされている領域の、どちらかへの書き込み
                    let addr = NAMETABLE_MIRROR_BASE_ADDR | pos;
                    self.vram[addr as usize] = data;
                    // 水平 or 垂直にミラーされている領域の、さっきとは違う側への書き込み
                    let addr = NAMETABLE_MIRROR_BASE_ADDR | pos.wrapping_add(offset);
                    self.vram[addr as usize] = data;
                }
            },
            0x3F00..=0x3F1F => {
                // パレット(またはそこへのミラー領域)への書き込み。
                
                // オリジナルとミラー領域の対応表は以下の通り。
                // これにより分かるのは、以下の2つの事実。
                // (1) アドレスの末尾2bitが 00 の場合、ミラー領域が存在する。
                // (2) (1)が成り立つ場合、4bit目を反転させればオリジナルとミラーを切り替え可能。

                // [オリジナルとミラーの対応表]
                // $3F00: 0011111100000000 (=$3F10)
                // $3F01: 0011111100000001
                // $3F02: 0011111100000010
                // $3F03: 0011111100000011
                // $3F04: 0011111100000100 (=$3F14)
                // $3F05: 0011111100000101
                // $3F06: 0011111100000110
                // $3F07: 0011111100000111
                // $3F08: 0011111100001000 (=$3F18)
                // $3F09: 0011111100001001
                // $3F0A: 0011111100001010
                // $3F0B: 0011111100001011
                // $3F0C: 0011111100001100 (=$3F1C)
                // $3F0D: 0011111100001101
                // $3F0E: 0011111100001110
                // $3F0F: 0011111100001111
                // $3F10: 0011111100010000 (=$3F00s)
                // $3F11: 0011111100010001
                // $3F12: 0011111100010010
                // $3F13: 0011111100010011
                // $3F14: 0011111100010100 (=$3F04)
                // $3F15: 0011111100010101
                // $3F16: 0011111100010110
                // $3F17: 0011111100010111
                // $3F18: 0011111100011000 (=$3F08)
                // $3F19: 0011111100011001
                // $3F1A: 0011111100011010
                // $3F1B: 0011111100011011
                // $3F1C: 0011111100011100 (=$3F0C)
                // $3F1D: 0011111100011101
                // $3F1E: 0011111100011110
                // $3F1F: 0011111100011111
                
                // 末尾2bitが 00 ならミラーへの反映が必要
                if (addr & 0b11) == 0 {
                    // オリジナルかミラーのどちらかへ書き込む
                    self.vram[addr as usize] = data;
                    // 4bit目を反転させて、先ほどとは逆の領域に書き込む
                    let addr = addr ^ (1 << 4);
                    self.vram[addr as usize] = data;
                }
            },
            _ => {
                // ミラーを考慮せずに書き込み
                self.vram[addr as usize] = data;
            }
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        self.vram[addr as usize]
    }
}

/*
VRAM Memory Map:
----------------------  ----------------------- --------------------
アドレス                用途                    物理的な位置
----------------------  ----------------------- --------------------
$0000-$0FFF $1000(4KB)  Pattern table 0         (CHR-ROM) 
$1000-$1FFF $1000(4KB)  Pattern table 1         (CHR-ROM)
$2000-$23FF $0400(1KB)  Nametable 0 (左上)      (専用RAM or ミラー)
$2400-$27FF $0400(1KB)  Nametable 1 (右上)      (専用RAM or ミラー)
$2800-$2BFF $0400(1KB)  Nametable 2 (左下)      (専用RAM or ミラー)
$2C00-$2FFF $0400(1KB)  Nametable 3 (右下)      (専用RAM or ミラー)
$3000-$3EFF $0F00(4KB)  Mirrors of $2000-$2EFF
$3F00-$3F1F $0020(32B)  Palette RAM indexes     (専用RAM)
$3F20-$3FFF $00E0(224B) Mirrors of $3F00-$3F1F
----------------------  --------------------------------------------
*/