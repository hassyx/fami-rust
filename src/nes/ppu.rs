//! NES PPU.

mod ppu_state;
mod vram;

use bitflags::bitflags;
use crate::nes::rom;
use crate::nes::ppu_databus::*;
use self::ppu_state::*;

/// スプライト用メモリ容量(bytes)
pub const SPR_RAM_SIZE: usize = 256;
/// 起動後、レジスタが外部からの呼びかけに応答を開始するまでのクロック数
const WARM_UP_TIME: u64 = 29658 * 3;

/*
[背景の描画：大まかな流れ]
・スクロール位置と、操作線の描画位置を考慮して、描画するピクセルの位置から、
  ネームテーブルの1つのタイルを割り出す。
・1個のネームテーブルのタイル(1バイト=0〜255)がパターンテーブルの
  インデックスになっているので、パターンテーブルのタイルを1個選択。
・パターンテーブルを見ることによって、8x8の各ピクセルが持っている2bitの
  情報を元に、どのパレットを利用すべきかがわかる。
・属性テーブルを読んで、パレット内のどの色を使うかを2bitの情報で割り出す。

[背景の描画：詳細]
・4枚のネームテーブルのうち1枚を選択。PPUCTRLの 0, 1 ビットで指定される。
  ネームテーブルの個々の要素は1バイトで、パターンテーブルへのインデックス
  (0-255)になっている。
  (ここではスクロール位置を考慮してネームテーブル中の描画対象タイルを決める)
・2枚あるパターンテーブルのうち1枚を選択。PPUCTRLの第4ビットで指定される。
  ネームテーブルのインデックスによって参照されているパターンテーブルを選択。
・ネームテーブルのインデックスから、描画に利用するパターンテーブルのタイルを選択。
  これによって描画すべきピクセルと、そのピクセルの色(のパレット)が判明する。
・描画に利用する、パレット内の色(全4色)を割り出す。描画に利用するネームテーブルが分かれば、
  属性テーブルも自動的に決まる。
  なお、属性テーブルは1バイト(8bit)で、1色(のindex)が2bitなので、
  「1バイト=4タイル分」を1まとめで色指定していることに注意。
*/

bitflags! {
    /// PPUCTRL
    pub struct CtrlFlags: u8 {
        /// Base NameTable Address.  
        /// (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
        const BASE_NAME_TABLE       = 0b0000_0011;
        /// PPUDATAでVRAMのデータを読み書きする際に増加するアドレス.  
        /// (0: add 1, going across; 1: add 32, going down)
        const VRAM_INCREMENT        = 0b0000_0100;
        /// Sprite pattern table address for 8x8 sprites.  
        /// (0: $0000; 1: $1000; ignored in 8x16 mode)
        const SPRITE_PATTERN_TABLE  = 0b0000_1000;
        /// Background pattern table address.  
        /// (0: $0000; 1: $1000)
        const BG_PATTERN_TABLE      = 0b0001_0000;
        /// Sprite size.  
        /// (0: 8x8 pixels; 1: 8x16 pixels)
        const SPRITE_SIZE           = 0b0010_0000;
        /// PPU master/slave select.  
        /// (0: read backdrop from EXT pins; 1: output color on EXT pins)
        const PPU_MASTER_SLAVE      = 0b0100_0000;
        /// Generate an NMI at the start of the vertical blanking interval  
        /// (0: off; 1: on)
        const NMI_ON_VBRANK         = 0b1000_0000;
    }
}

bitflags! {
    /// PPUMASK
    pub struct MaskFlags: u8 {
        /// Greyscale
        /// (0: normal color, 1: produce a greyscale display)
        const GRAYSCALE             = 0b0000_0001;
        /// (0:画面上の左端8ピクセルでBGを隠す)
        const SHOW_BG_LEFTMOST      = 0b0000_0010;
        /// (0:画面上の左端8ピクセルでスプライトを隠す)
        const SHOW_SPRITE_LEFTMOST  = 0b0000_0100;
        /// (0:BGを非表示)
        const SHOW_BG               = 0b0000_1000;
        /// (0:スプライトを非表示)
        const SHOW_SPRITE           = 0b0001_0000;
        /// (1:赤を強調)
        const EMPHASIZE_RED         = 0b0010_0000;
        /// (1:緑を強調)
        const EMPHASIZE_GREEN       = 0b0100_0000;
        /// (1:青を強調)
        const EMPHASIZE_BLUE        = 0b1000_0000;
    }
}

bitflags! {
    /// PPUSTATUS
    pub struct StatusFlags: u8 {
        /// スキャンライン上のスプライトが8個以下なら1、9個以上なら1。
        /// スプライトの評価中に設定され、pre-render-lineの1ドット目
        /// (第2バイト目)でクリアされる。
        const SPRITE_OVERFLOW   = 0b0010_0000;
        /// スプライト0ヒット判定。ヒットしていれば1。
        /// 一度設定されると、次のVBlankが終わるまでクリアされない。
        const SPRITE_ZERO_HIT   = 0b0100_0000;
        /// VBlank発生時に1。
        const VBLANK_OCCURRED   = 0b1000_0000;
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
    pub databus: u8,
    // TODO: PPUSCROLLとPPUADDRのトグルを実現する隠しレジスタを実装する。
}

impl Registers {
    /// PPUSTATUSの読み取りと、各種情報のリセット
    pub fn read_status(&self) -> u8 {
        // TODO: 読み込み時に以下が発生。
        // ・ラッチの状態をクリア。    
        // ・statusの7bit目を0にクリア。
        0
    }
}

pub struct Ppu {
    state: &'static PpuState,
    regs: Registers,
    /// スプライト用のメモリ(256バイト)。
    /// OAM(Object Attribute Memory)ともいう。
    /// VRAMと違い、特別な対応が必要ないのでベタな配列として扱う。
    spr_ram: Box<[u8]>,
    /// VRAMへのアクセスを司るコントローラ
    vram: Box<vram::MemCon>,
    clock_counter: u64,
    reset_requested: bool,
}

impl Ppu {
    pub fn new(rom: &rom::NesRom) -> Ppu {
        let mut my = Ppu {
            state: &STATE_IDLING,
            regs: Default::default(),
            spr_ram: Box::new([0; SPR_RAM_SIZE]),
            vram: Box::new(vram::MemCon::new(rom.mirroring_type())),
            clock_counter: 0,
            reset_requested: false,
            //fn_step: Ppu::prepare_step,
            //state: Default::default(),
        };
        
        // CHR-ROM(パターンテーブル) を VRAM に展開。
        // VRAM上にCHR-ROMを置く領域は16KB分存在するが、CHR-ROMが1枚(8KB)のみの
        // ROMがある。その場合でも1枚分を追加でコピー済みなので、ここで一括転送可能。
        // TODO: マッパーによってはCHR-ROMが2枚以上載っている可能性あり。
        let chr_rom = rom.chr_rom();
        let len = rom::CHR_ROM_UNIT_SIZE;
        if chr_rom.len() >= len {
            my.vram.raw_write(0, &chr_rom[0..len]);
        }

        return my
    }

    pub fn power_on(&mut self) {
        // 電源ON時のPPU状態
        // https://wiki.nesdev.org/w/index.php/PPU_power_up_state

        // レジスタの初期化
        self.regs.ctrl = 0;
        self.regs.mask = 0;
        self.regs.status = 0;
        self.regs.oam_addr = 0;
        self.regs.databus = 0;
        self.regs.scroll = 0;
        self.regs.addr = 0;
        self.regs.data = 0;
        
        self.signal_reset();

        // TODO: 描画位置を左上に設定
    }

    fn signal_reset(&mut self) {
        // リセットの発生はVBLANKフラグが落ちるタイミングまで遅延される
        self.reset_requested = true;
    }

    /// リセット処理。VBLANKフラグが落ちた際に実行される。
    fn reset(&mut self) {
        /*
        リセット時の挙動：
        - PPUCTRL、PPUMASK、PPUSCROLL、PPUADDR、PPUSCROLL / PPUADDRラッチ、
          およびPPUDATA読み取りバッファをクリアする内部リセット信号がある。
          (PPUSCROLLとPPUADDRをクリアすることは、VRAMアドレスラッチ(T)と
          細かいXスクロールをクリアすることに相当する。VRAMアドレス自体(V)は
          クリアされないので注意。)
        - このリセット信号は、リセット時に設定され、VBlank、スプライト0、
          およびオーバーフローフラグをクリアするのと同じ信号によってVBlankの
          最後にクリアされる。つまりリセット発生〜VBlank終了時までの、
          上記レジスタへの書き込みは無視される。
        */
        self.regs.ctrl = 0;
        self.regs.mask = 0;
        self.regs.status = 0;
        self.regs.databus = 0;
        self.regs.scroll = 0;
        self.regs.data = 0;

        // TODO: アドレス用の隠しレジスタも初期化する。
        // というか、そもそも「隠しレジスタの内容が$2005, $2006の内容」なのだろうか？
        
        self.signal_reset();
    }
    
    /// PPUを1クロック進める。
    /// NMI(vblank)が発生した場合はtrueを返す。
    pub fn step(&mut self) -> bool {
        self.clock_counter += 1;
        //self.state.counter += 1;
        (self.state.step)(self);
        // print_ppu_state!(self);
        false
    }

    fn render() {
        // TODO: CPUとPPUの1クロックあたりに描画可能なピクセル数

        // TODO: PPUはCPUと独立したクロックカウンターを持ち、
        // そのクロックを基準として動く(CPUに合わせて3倍にはしない)

        // 割り込み発生した場合は戻り値として返す。

        // PPUはROMによる初期化処理の前から動いている。
        // レジスタの初期値は？



        // [NTSCの基礎知識]
        // 縦横比は 3:4。走査線は525本。書き換え頻度は60Hz。
        // ただし1回に書き換えられる走査線はこの半分で、インターレースの飛び越し走査を行う。
        // 525本のうち見切れる部分があるので、有効垂直解像度は486本。
        // 水平解像度は約330本相当。
        // インターレースなので、(30Hz x 2) で1画面を描画する。
        // 1画面を「フレーム」と呼ぶ。
        // 1画面の描画に2回の走査が必要で、1回の走査(262.5本分)のことを「1フィールド」という。
        // 走査線が525本なので、2では割り切れない。(525 / 2 = 262.5)

        // [NESの描画方法(いわゆる「240P」)について]
        // 垂直同期パルスのタイミングを変更せずに、同じラインに描き続ける。
        // NTSCの標準からは外れた手法。

        // [NESが管理する画面の構成]
        // https://wiki.nesdev.org/w/index.php/Overscan
        // NESの(物理的ではなく内部的な)解像度は、256x240。240がY軸(スキャンライン)。
        // 実際にはオーバースキャンで確実に表示されない走査線が上下に (11x2)個あるので、
        // 実際に描画する走査線は 262本 となる。
        // 262本はNTSCの525本の約半分しかないが、NESではインターレースの飛び越し走査を行わず、
        // 歯抜けの状態で、常に同じスキャンラインへ60Hzで書き込んでいる(いわゆる「240P」)。

        // オーバースキャンを考慮すると、走査線の縦240本のうち実際に表示されるのは中央部の 224本 程度。
        // オーバースキャンのマージンを最大に取ると、224x192 程度まで狭まる。
        
        // NTSCのスキャンライン1行分に要する時間を考慮すると、PPUは1スキャンラインごとに
        // 280ピクセルを描画するための猶予がある。
        // PPUは、280のうち中央の256を実際に描画し、残りを左右の空白(12+12)に充てる。
        // 空白は背景色(カラーパレットの$3F00)が適用される。

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
        //      2バイトがフェッチされるが、この目的は不明。エミュレーターでは実装しなくていい。

        // [NameTable=BGの描画処理]
        // まず、PPUSCROLLによるスクロール量を考慮して、描画するピクセルがNameTable上のどの位置に該当するかを割り出す。
        // 
    }
}

impl PpuDataBus for Ppu {
    fn write(&mut self, reg_type: PpuRegs, data: u8) {
        (self.state.write)(self, reg_type, data);
    }
    
    fn read(&mut self, reg_type: PpuRegs) -> u8 {
        (self.state.read)(self, reg_type)
    }

    fn dma_write(&mut self, data: u8) {
        self.dma_write(data);
    }
}