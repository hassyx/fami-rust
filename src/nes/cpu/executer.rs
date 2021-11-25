//! Instruction executer.

use super::Cpu;
use super::decoder::AddrMode;

// TODO: 命令の基本パターンを調べる。
// 例えば、OPコード、メモリ1バイトフェッチ、はそれぞれ1クロックサイクルのはず。
// それら以外の、特別な条件をすべてリストアップ。
// あと、パイプラインが影響したりする？特に条件分岐の先読み。

impl Cpu {
    /*
    /// ORA: レジスタAとメモリをORしてAに格納。
    pub fn ora(&mut self, addr_mode: AddrMode) {
        let val = self.fetch();
        let result = match addr_mode {
            AddrMode::Immediate => {
                self.
            },
        }
    }
    */

/*
    /// 不正なアドレッシングモード。
    Invalid,
    /// Aレジスタに対して演算を行い、Aレジスタに格納する。
    Accumulator,
    /// オペランドの16bitの即値との演算。
    Immediate,
    /// オペランドで16bitのアドレスを指定し、参照先の8bitの値と演算を行う。
    Absolute,
    /// オペランドで16bitのアドレス(ただし0-255の範囲)を指定し、参照先の8bitの値と演算を行う。
    ZeroPage,
    /// オペランドで指定した16bitのアドレスに、レジスタXの値を足して、
    /// そのアドレスが指す8bitの値に対して演算を行う。
    IndexedAbsolute_X,
    /// オペランドで指定した16bitのアドレスに、レジスタYの値を足して、
    /// そのアドレスが指す8bitの値に対して演算を行う。
    IndexedAbsolute_Y,
    /// オペランドで指定した16bitのアドレス(ただし範囲は0-255)に、レジスタX(一部の命令ではY)を加算して、
    /// そのアドレスが指す8bitの値に対して演算を行う。
    /// 算出したアドレスがゼロページ(0-255)を超過する、しないに関わらず、常に下位8bitの値しか見ない。
    IndexedZeroPage_X,
    /// オペランドで指定した8bitの値に、レジスタXの値を足して、ゼロページ内のアドレスを得る。
    /// 次に、このアドレスの指す8bitを下位アドレス、アドレス+1 の指す内容を上位8bitとして、
    /// 16bitの最終アドレスを得る。この最終アドレスの指す先の、8bitの値に対して操作を行う。
    /// なお、1段階目と2段階目で算出したアドレスが8bitを越える、越えないに関わらず、常に下位の8bitのみを見る。
    IndexedIndirect_X,
    /// オペランドで指定した8bitのアドレスを下位アドレス、アドレス+1 の指す内容を上位8bitとして、
    /// 16bitのアドレスを得る。このアドレスに、レジスタYの値を足して、最終アドレスを得る。
    /// 最終アドレスの指す先の8bitの値に対して操作を行う。
    /// なお、1段階目と2段階目で算出したアドレスが8bitを越える、越えないに関わらず、常に下位の8bitのみを見る。
    IndirectIndexed_Y,
*/
}
