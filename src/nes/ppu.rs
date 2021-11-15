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

    pub fn step(&mut self, clk_cnt: u64) -> u32 {

        return 0;

        // [NTSCの基礎知識]
        // 縦横比は 3:4。走査線は525本。書き換え頻度は60Hz。
        // ただし1回に書き換えられる走査線はこの半分で、インターレースの飛び越し走査を行う。
        // 525本のうち見切れる部分があるので、有効垂直解像度は486本。
        // 水平解像度は約330本相当。
        // インターレースなので、(30Hz x 2) で1画面を描画する。
        // 1画面を「フレーム」と呼ぶ。
        // 1画面の描画に2回の走査が必要で、1回の走査(262.5本分)のことを「1フィールド」という。
        // 走査線が525本なので、2では割り切れない。(525 / 2 = 262.5)

        // [NESが管理する画面の構成]
        // https://wiki.nesdev.org/w/index.php/Overscan
        // NESの(物理的ではなく内部的な)解像度は、256x240。240がY軸(スキャンライン)。
        // 実際にはオーバースキャンで確実に表示されない走査線が上下に (11x2)個あるので、
        // 実際に描画する走査線は 262本 となる。
        // 262本はNTSCの525本の約半分しかないが、インターレースの飛び越し走査を行わず、
        // 歯抜けの状態で、常に同じスキャンラインへ60Hzで書き込んでいる(いわゆる240P)。
        
        // オーバースキャンを考慮すると、走査線の縦240本のうち実際に表示されるのは中央部の 224本 程度。
        // オーバースキャンのマージンを最大に取ると、224x192 程度まで狭まる。
        
        // NTSCのスキャンライン1行分に要する時間を考慮すると、PPUは1スキャンラインごとに
        // 280ピクセルを描画するための猶予がある。
        // PPUは、280のうち中央の256を実際に描画し、残りを左右の空白(12+12)に充てる。
        // 空白は背景色(カラーパレットの$3F00)が適用される。

        // [NESの描画方法(いわゆる「240P」)について]
        // 垂直同期パルスのタイミングを変更せずに、同じラインに描き続ける。
        // NTSCの標準からは外れた手法。

        // NESのPPUはHBLANK(水平帰線区間)の割り込みを発生させないため、
        // ソフト側が自力でスプライト0ヒットフラグ(PPUSTATUS:$2002の bit 6)を
        // ポーリングし、実装する必要がある。
        // MMCによっては、PPUのアドレスライン・データラインを追跡し、
        // HBLANKを発生させるカセットもある。(MMC3など)
        
        // [1画面を描画するまでの処理内容]
        // => 実際にはオーバースキャン分描画がズレているので、PPUが最初に出力するピクセルは、
        //    画面上の位置としては(12x11)になる。
        //
        // [line 260.5-0.5]
        //      描画は行わない。最下位のスキャンラインから最上位に戻る期間。
        //      次のラインの最初の8ピクセル分を先読みしている。
        //      280-304ピクセルの間に、レンダリングが有効になっている場合、
        //      垂直スクロールビットがリロードされる。
        //      TODO: 奇数フレームか偶数フレームかで処理が異なる。
        // [line 0-239]
        //      可視のスキャンライン。描画を行う。基本的にこの間PPUを触ってはいけない。
        // [line 240]
        //      アイドル。PPUは何もしない。PPUに触っても安全だが、VBlankはまだ発生していない。
        // [line 241-260]
        //      VBlank期間。line 241の1クロックサイクル目、つまり第2サイクルでVBlankが発生する。
        //      この期間はPPUがメモリにアクセスを行わない。
        // [line 260.5-0.5]
        //      最初に戻る。

        // [1スキャンライン内の、クロックサイクルごとの処理内容]
        // (1スキャンライン=341クロックサイクル)
        // [0 cc]  
        //      アイドル。PPUは何も行わない。
        // [1-256 cc]
        //      PPUはメモリからデータを読みながら、1ピクセルずつラインを埋めていく。
        //      描画の裏で、以下の4つのテーブルから、それぞれ 2cc かけて 1バイトずつメモリを読む。
        //        - Name Table
        //        - Attribute Table
        //        - Pattern Table(Low)
        //        - Pattern Table(high)
        //      8bit を書いている間に 8bit を読むので、PPUは途切れず描画を行うことができる。
        //      スプライトの0ヒットはここでチェックされる。
        // [257-320 cc]
        //      次のスキャンラインに書くスプライトのデータをフェッチする。
        // [321-336 cc]
        //      次のスキャンラインに書く最初のタイル2個分を先読みする。
        // [337-340 cc]
        //      2バイトがフェッチされますが、この目的は不明。エミュレーターでは実装しなくていい。

        
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
