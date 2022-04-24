//! Utilities.

use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum Error {
    Msg(String),
}

impl Error {
    pub fn new(msg: String) -> Box<dyn std::error::Error> {
        Box::new(Error::Msg(msg))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Error::Msg(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}

pub fn err_exit(msg: &str) -> ! {
    eprintln!("{}", msg);
    std::process::exit(1);
}

pub fn panic_write_to_read_only_area(addr: u16, data: u8) -> ! {
    panic!("Error: Write to read-only area. addr={:#06X}, data={:#04X}", addr, data)
}

pub fn make_addr(high: u8, low: u8) -> u16{
    (high as u16) << 8 | (low as u16)
}