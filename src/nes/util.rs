//! Utilities.

use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum Error {
    Msg(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Error::Msg(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for Error {}

// TODO: 基本この関数からの出力は、verboseモード時のみ表示させたい。
// ユーザーに表示するエラーを決めるため、エラーの大分類を指定する引数を追加すればいいのか？
pub fn err_exit(msg: &str) -> ! {
    eprintln!("{}", msg);
    std::process::exit(1);
}

pub fn make_addr(high: u8, low: u8) -> u16{
    (high as u16) << 8 | (low as u16)
}