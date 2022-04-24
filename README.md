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
1. Dump the cartridge yourself, Or get a public domain ROM(\*). Supports NES 2.0 format.
2. Run the rom image files. ex. `cargo run supermario.nes` or `fami-rust supermario.nes`.
3. An empty window will appear (because graphics are not implemented yet). Application keeps running without graphics.

(\*) [nestest.nes](http://nickmass.com/images/nestest.nes) is available for comprehensive testing of all 6502 instructions.
