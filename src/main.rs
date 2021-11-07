mod nes;
use nes::rom;
use nes::util;
use nes::cpu;

extern crate piston_window;
use piston_window::*;
fn main() {
    let path = "./ignores/donkeykong.nes";
    let rom = load_rom(path);
    let mut cpu = cpu::CPU::default();
    cpu.attach_rom(rom);
    cpu.power_on();

    let mut window: PistonWindow = WindowSettings::new("Fami-Rust", (640, 480))
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| { panic!("Failed to build PistonWindow: {}", e) });
    
    while let Some(e) = window.next() {
        window.draw_2d(&e, |_c, g, _d| {
            clear([0.5, 1.0, 0.5, 1.0], g);
        });
    }
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

/*
mod nes;
use nes::rom;
use nes::util;
use nes::cpu;

fn main() {
    let path = "./ignores/donkeykong.nes";
    let rom = load_rom(path);
    let mut cpu = cpu::CPU::default();
    cpu.attach_rom(rom);
    cpu.power_on();
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
*/