mod nes;

use nes::rom;
use nes::util;
use nes::cpu;
use nes::ppu;

extern crate piston_window;
extern crate image;

use piston_window::*;

fn main() {
   
    // ROMをロード
    let path = "./ignores/donkeykong.nes";
    let rom = load_rom(path);

    // PPUを初期化
    let ppu_regs = ppu::Registers::default();
    // VRAM側に、PPUのレジスタとROMのCHR-ROM領域をマッピングする。
    let mut ppu = ppu::PPU::new(&ppu_regs, &rom);
    ppu.power_on();

    // CPUを初期化
    let cpu_regs = cpu::Registers::default();
    // RAMにPPUのレジスタをマッピングする。
    let mut cpu = cpu::CPU::new(&cpu_regs, &ppu_regs, &rom);
    cpu.power_on();
    
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
            let cpu_clk = cpu.exec();
            // TODO: clock_cycle * clock_freq 分、待機する。

            // TODO: 3回に1回、ppuが動作する
            let ppu_clk = ppu.exec();

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