mod nes;

use std::cell::RefCell;
use std::rc::Rc;

use nes::ppu_databus;
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
    std::env::set_var("RUST_LOG", "debug");
    env_logger::builder()
        .format_timestamp(None)
        .init();

    // ROMをロード
    let path = "./ignores/donkeykong.nes";
    let rom: Box<NesRom> = load_rom(path);

    // PPUを初期化
    // VRAMにROMのCHR-ROM領域をマッピングする。
    let ppu = Rc::new(RefCell::new(Ppu::new(&rom)));
    ppu.borrow_mut().power_on();

    // RAMを初期化
    let ppu_databus = Box::new(ppu_databus::DataBus::new(Rc::clone(&ppu)));
    let ram = MemCon::new(ppu_databus);

    // CPUを初期化
    let mut cpu = Cpu::new(&rom, Box::new(ram));
    cpu.power_on();
    
    const WINDOW_X: u32 = 640;
    const WINDOW_Y: u32 = 480;

    // Create window.
    let mut window: PistonWindow = 
        WindowSettings::new("Fami-Rust", (WINDOW_X, WINDOW_Y))
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| { panic!("Failed to build PistonWindow: {}", e) });
    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into()
    };
    let mut screen = image::ImageBuffer::new(WINDOW_X, WINDOW_Y);
    let mut texture: G2dTexture = Texture::from_image(
        &mut texture_context,
        &screen,
        &TextureSettings::new()
    ).unwrap();

    let mut cpu_counter: u8 = 3;

    // Start main loop.
    while let Some(e) = window.next() {
        if let Some(_) = e.render_args() {

            // 3回に1回、CPUが動作する
            if cpu_counter >= 3 {
                // CPUの処理を進める
                cpu.step();
                cpu_counter = 0;
            }

            if ppu.borrow_mut().step() {
                cpu.trigger_nmi();
            }

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
        // キーが押されていたら割り込みトリガをONにする。
        if let Some(Button::Keyboard(key)) = e.release_args() {

        }

        if let Some(Button::Keyboard(key)) = e.press_args() {

        }

        cpu_counter += 1;
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
