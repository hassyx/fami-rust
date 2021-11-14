//! NES PPU.

use crate::nes::rom;
use crate::nes::vram;

/// スプライト用メモリ容量(bytes)
pub const SPR_RAM_SIZE: usize = 256;

pub struct Ppu {
    pub regs: Registers,
    /// スプライト用のメモリ(256バイト)
    /// VRAMと違い、特別な対応が必要ないのでベタな配列として扱う。
    spr_ram: Box<[u8]>,
    /// VRAMへのアクセスを司るコントローラ
    vram: Box<vram::MemCon>,
    clock_count: u64,
}

impl Ppu {
    pub fn new(rom: &rom::NesRom) -> Ppu {
        let mut my = Ppu {
            regs: Default::default(),
            spr_ram: Box::new([0; SPR_RAM_SIZE]),
            vram: Box::new(vram::MemCon::new()),
            clock_count: 0,
        };
        
        {
            // CHR-ROM を VRAM に展開
            let chr_rom = rom.chr_rom();
            let len = rom::CHR_ROM_UNIT_SIZE;
            if chr_rom.len() >= len {
                my.vram.raw_write(0x0000, &chr_rom[0..len]);
            }
        }

        return my
    }

    /// 起動後、29658クロックに「到達した」時点から書き込みを許可する。
    pub fn is_ready(clock_count: u64) -> bool {
        clock_count >= 29658
    }

    pub fn power_on(&mut self) {
        // 起動時のPPU状態
        // https://wiki.nesdev.org/w/index.php/PPU_power_up_state

        // スキャンラインを0に戻す
        //
    }

    pub fn render(&mut self, clk_cnt: u64) -> u32 {

        return 0;

        // NTSCの基礎知識：
        // 縦横比は 3:4。走査線は525本。書き換え頻度は60Hz。
        // 525本のうち見切れる部分があるので、有効垂直解像度は486本。
        // 水平解像度は約330本。
        // インターレースなので、(30Hz x 2) で1画面を描画する。
        // 1画面を「フレーム」と呼ぶ。
        // 1画面の描画に2回の走査が必要で、1回の走査(262.5本分)のことを「1フィールド」という。
        // 走査線が525本なので、2では割り切れない。(525 / 2 = 262.5)

        // NESが管理する画面の構成
        // 1スキャンラインは 341 pixel。
        // スキャンラインは全部で 262本。

        // NESの描画方法(いわゆる「240P」)について
        // 垂直同期パルスのタイミングを変更せずに、同じラインに描き続ける。
        // NTSCの標準からは外れた手法。

        // NESのPPUはHBLANK(水平帰線区間)の割り込みを発生させないため、
        // 

        // スキャンラインの描画について(0-261)
        // 260.5-0.5:   何もしない。最下位のスキャンラインから最上位に戻る期間。
        //              TODO: 奇数フレームか偶数フレームかで処理が異なる。
        // 0-239:       可視のスキャンライン。
        // 240:         アイドル。PPUは何もしない。
        // 

        // line 1-240: ユーザーから見えるスキャンライン.
        // line 241: NTSCの1画面を描き終わった。PPU的にはアイドル時間.
        // line 242-261: vertical blanking line. いわゆるVBLANK区間.
        // line 261: 

        // 1クロックサイクルで1ピクセルを描画する
        // 341クロックサイクルで1スキャンラインを描画する。

        // TODO: 必要であれば、偶数・奇数フレームのクロックサイクル数の違いに対応する



        // 水平帰線区間は？？？？ NTSCと同じだろう。
        // 垂直帰線区間は、XXXから始まり、XXXで終わる。


        // VRAM(とCHR-ROM)へのアクセス
        // 1.ネームテーブル読み込み
        // 2.アトリビュート読み込み
        // 3.タイルパターン読み込み(LOW 側プレーン)
        // 4.タイルパターン読み込み(HIGH 側プレーン)
    }

    /// 割り込み
    pub fn interrupt(&self) {
        // TODO: CPU側にVBlankを投げる必要あり。
    }
}

#[derive(Default)]
pub struct Registers {
    /// PPUCTRL($2000): 書き込み専用。PPU制御用のフラグレジスタ。
    pub ctrl: u8,
    /// PPUMASK($2001): 書き込み専用。マスク処理用のレジスタ。
    pub mask: u8,
    /// PPUSTATUS ($2002): 読み込み専用。ステータスレジスタ。
    pub status: u8,
    /// OAMADDR ($2003): 書き込み専用。OAM(SPR-RAM)への書き込み先アドレス設定用のレジスタ。
    pub oam_addr: u8,
    /// OAMDATA ($2004): 読み書き可能。OAM(SPR-RAM)への読み書きレジスタ。
    pub oam_data: u8,
    /// PPUSCROLL ($2005): 書き込み専用。スクロール位置変更用レジスタ。
    pub scroll: u8,
    /// PPUADDR ($2006): 書き込み専用。VRAMへの書き込み位置の指定用レジスタ。
    pub addr: u8,
    /// PPUDATA ($2007): 読み書き可能。VRAMへの書き込みと読み込み用レジスタ。  
    pub data: u8,
    /// OAMDMA ($4014): 書き込み専用。OAM(SPR-RAM)へのDMA転送に使用する、
    /// source(CPU側のRAM)側のアドレスを指定するレジスタ。  
    pub oam_dma: u8,
    /// CPUとPPUのデータ転送に利用するバス。実体は8bitのラッチ。
    /// PPUのレジスタへ、CPU側から読み書きを行った場合に更新される。
    pub latch: u8,
}

/*
pub struct Reg {
    data: u8,
    flags: u8,
}

impl Reg {
    const IS_READABLE: u8 = 0b0000_0001;
    const IS_WRITABLE: u8 = 0b0000_0010;
    
    /// 外部デバイスからのレジスタへの書き込み
    fn ext_write() {

    }
}
*/
