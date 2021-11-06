mod nes;
use nes::rom;
use nes::util;

fn main() {
    let path = "./ignores/donkeykong.nes";
    let rom = match rom::load_from_file(&path) {
        Ok(bin) => bin,
        Err(err) => {
            // TODO:エラー時のメッセージをユーザーフレンドリーに
            util::err_exit(&err.to_string());
        },
    };

    
}

/*
fn load_rom() -> Box<nes::rom::NesRom> {
    let rom = match nesrom::load_from_file(&path) {
        Ok(bin) => bin,
        Err(err) => {
            // TODO:エラー時のメッセージをユーザーフレンドリーに
            util::err_exit(&err.to_string());
        },
    };
    rom
}
*/