use serde_msgpack::decode;
use serde_msgpack::encode;

use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
  Io(io::Error),
  MsgPackEnc(encode::Error),
  MsgPackDec(decode::Error),
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    match *self {
      Error::Io(ref e) => write!(f, "{}", e),
      Error::MsgPackEnc(ref e) => write!(f, "{}", e),
      Error::MsgPackDec(ref e) => write!(f, "{}", e),
    }
  }
}
