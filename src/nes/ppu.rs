//! NES PPU.

mod ppu_state;
mod vram;

use bitflags::bitflags;

use crate::nes::rom;
use crate::nes::ppu::ppu_state::*;

/// スプライト用メモリ容量(bytes)
pub const SPR_RAM_SIZE: usize = 256;
/// 起動後、レジスタが外部からの呼びかけに応答を開始するまでのクロック数
const WARM_UP_TIME: u64 = 29658;

/*
TODO: 実装に必要な情報
◯初期化時に何が起こるか？
◯レジスタの初期値は何か？
・書き込み/読み込み禁止のレジスタに書き込み/読み込みした場合の振る舞い
・ラッチの2回書き込みを実装すること
・スプライトの評価とは？いつ行われるのか？(PPUSTATUS)
・pre-render-lineとは何か？(PPUSTATUS)
・Base NameTable Address には何の意味がある？
・
*/

bitflags! {
    /// PPUCTRL
    pub struct CtrlFlags: u8 {
        /// Base NameTable Address  
        /// (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
        const SPRITE_OVERFLOW   = 0b0000_0011;
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
        const VBLANK            = 0b1000_0000;
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
    /// PPUSCROLLとPPUADDRに 2バイト分の書き込みを行うために存在する。
    pub latch: u8,
}

impl Registers {
    /// PPUSTATUSの読み取り。
    pub fn read_reg_status() {
        // TODO: 読み込み時に以下が発生。
        // ・ラッチの状態をクリア。    
        // ・statusの7bit目を0にクリア。
    }
}

pub struct Ppu {
    pub regs: Registers,
    /// スプライト用のメモリ(256バイト)。
    /// OAM(Object Attribute Memory)ともいう。
    /// VRAMと違い、特別な対応が必要ないのでベタな配列として扱う。
    spr_ram: Box<[u8]>,
    /// VRAMへのアクセスを司るコントローラ
    vram: Box<vram::MemCon>,
    clock_counter: u64,
    fn_step: FnState,
    state: TmpState,
}

impl Ppu {
    pub fn new(rom: &rom::NesRom) -> Ppu {
        let mut my = Ppu {
            regs: Default::default(),
            spr_ram: Box::new([0; SPR_RAM_SIZE]),
            vram: Box::new(vram::MemCon::new()),
            clock_counter: 0,
            fn_step: Ppu::prepare_step,
            state: Default::default(),
        };
        
        {
            // CHR-ROM を VRAM に展開。
            // VRAM上にCHR-ROMを置く領域は2KB分存在するが、CHR-ROMが1枚(1024Kバイト)しか
            // 存在しないROMがある。その場合でも1枚分をコピー済みなので、ここで一括転送可能。
            // TODO: マッパーによってはCHR-ROMが複数載っている可能性あり。
            let chr_rom = rom.chr_rom();
            let len = rom::CHR_ROM_UNIT_SIZE;
            if chr_rom.len() >= len {
                my.vram.raw_write(0, &chr_rom[0..len]);
            }
        }

        return my
    }

    /// 起動後、PPU換算で29658クロックに「到達した」時点から書き込みを許可する。
    pub fn is_ready(&self) -> bool {
        self.clock_counter >= WARM_UP_TIME
    }

    pub fn power_on(&mut self) {
        // 起動時のPPU状態
        // https://wiki.nesdev.org/w/index.php/PPU_power_up_state

        // スキャンラインを0に戻す

        // レジスタ等の初期化
        // TODO: 規定クロック経過後はまた違う値を持つ可能性がある
        self.regs.ctrl = 0;
        self.regs.mask = 0;
        self.regs.status = 0;
        self.regs.oam_addr = 0;
        // !!!実装中!!!

        // TODO: ROM

        self.fn_step = Ppu::prepare_step;
        self.state = Default::default();
    }

    pub fn step(&mut self) -> u32 {

        // TODO: PPUはCPUと独立したクロックカウンターを持ち、
        // そのクロックを基準として動く(CPUに合わせて3倍にはしない)

        // 割り込み発生した場合は戻り値として返す。
        return 0;

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
        //      PPUはメモリからデータを読みなが1ら、1ピクセルずつラインを埋めていく。
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

