mod nes;
use nes::rom;
use nes::util;

fn main() {
    let path = "./ignores/donkeykong.nes";
    let rom = load_rom(path);
}

fn load_rom(path: &str) -> Box<rom::NesRom> {
    match rom::load_from_file(&path) {
        Ok(bin) => bin,
        Err(err) => {
            // TODO:エラー時のメッセージをユーザーフレンドリーに
            util::err_exit(&err.to_string());
        },
    }
}
