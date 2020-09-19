use std::io;
use std::string::FromUtf8Error;

#[derive(Debug, Error)]
pub enum Error {
  /// Error decoding websocket text frame Utf8
  #[error("error decoding websocket text frame as UTF-8")]
  Utf8(#[source] FromUtf8Error),
  /// Error decoding Json
  //JsonDecode(rustc_serialize::json::DecoderError),
  /// Error parsing Json
  //JsonParse(rustc_serialize::json::ParserError),
  /// Error encoding Json
  //JsonEncode(rustc_serialize::json::EncoderError),
  /// Errors that do not fit under the other types, Internal is for EG channel errors.
  #[error("internal error")]
  Internal(String),
}

impl From<io::Error> for Error {
  fn from(err: io::Error) -> Error {
    Error::Internal(format!("{:?}", err))
  }
}

impl From<FromUtf8Error> for Error {
  fn from(err: FromUtf8Error) -> Error {
    Error::Utf8(err)
  }
}
