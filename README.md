# fami-rust:
*fami-rust* is a Family Computer(NES) emulator written in Rust.

`Notice: Currently only CPU(6502) works. PPU(Picture Processing Unit) and APU(Audio Processing Unit) are still under development.`

## Supported Platforms
Windows / Mac / Linux.

## License
MIT License

## Setup
`cargo build`

## Usage
1. If you own a NES cartridge and rom-damper, dump it to get a rom image. For your reference, I use [FC DUMPER](https://www.gamebank-web.com/).
2. If you don't own these, you can use a copyright-friendly ROM images(\*). 
3. Run the rom image file. ex: `cargo run supermario.nes` or `fami-rust supermario.nes`.
4. An empty window will appear (because graphics are not implemented yet). Application keeps running without graphics.
5. You can see that all CPU states are output to the console every clock.

(\*) [nestest.nes](http://nickmass.com/images/nestest.nes) is available for comprehensive testing of all 6502 instructions.
