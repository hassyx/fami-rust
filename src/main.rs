mod nes;

use std::cell::RefCell;
use std::clone;
use std::rc::Rc;

use nes::rom::NesRom;
use nes::rom;
use nes::util;
use nes::cpu::Cpu;
use nes::ppu::Ppu;
use nes::mem::MemCon;

extern crate piston_window;
extern crate image;

use piston_window::*;

fn main() {
    let clock_count: u64 = 0;

    // ROMをロード
    let path = "./ignores/donkeykong.nes";
    let rom = load_rom(path);

    // PPUを初期化
    // VRAM側に、PPUのレジスタとROMのCHR-ROM領域をマッピングする。
    let ppu = Rc::new(RefCell::new(Ppu::new(&rom)));
    ppu.borrow_mut().power_on();

    // RAMを初期化
    let ram = MemCon::new(Rc::clone(&ppu));

    // CPUを初期化
    let mut cpu = Cpu::new(&rom, Box::new(ram), clock_count);
    cpu.power_on(clock_count);
    
    const window_x: u32 = 640;
    const window_y: u32 = 480;

    // Create window.
    let mut window: PistonWindow = 
        WindowSettings::new("Fami-Rust", (window_x, window_y))
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| { panic!("Failed to build PistonWindow: {}", e) });
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into()
    };
    let mut screen = image::ImageBuffer::new(window_x, window_y);
    let mut texture: G2dTexture = Texture::from_image(
        &mut texture_context,
        &screen,
        &TextureSettings::new()
    ).unwrap();

    // Start main loop.
    while let Some(e) = window.next() {
        if let Some(_) = e.render_args() {
            // CPUの処理を進める
            let cpu_clk = cpu.step(clock_count);
            // TODO: clock_cycle * clock_freq 分、待機する。

            // TODO: 3回に1回、ppuが動作する
            let ppu_clk = ppu.borrow_mut().render(clock_count);

            // 試しに点を打ってみる
            screen.put_pixel(100, 100, image::Rgba([255, 127, 127, 255]));

            texture.update(&mut texture_context, &screen).unwrap();
            window.draw_2d(&e, |c, g, device| {
                texture_context.encoder.flush(device);
                image(&texture, c.transform, g);
                // clear([0.5, 1.0, 0.5, 1.0], g);
            });
        }

        // 以下キーイベント処理。
        // 場合によってはキーイベントを吸い上げても、
        if let Some(Button::Keyboard(key)) = e.release_args() {

        }

        if let Some(Button::Keyboard(key)) = e.press_args() {

        }
    }
}

fn load_rom(path: &str) -> Box<NesRom> {
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