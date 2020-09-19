#![recursion_limit = "1024"]

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate async_stream;
#[macro_use]
extern crate thiserror;

pub mod chan;
pub mod error;
pub mod event;
pub mod message;
pub mod socket;

mod websocket;

pub use chan::Channel;
pub use error::Error;
pub use event::Event;
pub use event::PhoenixEvent;
pub use message::Message;
pub use socket::Phoenix;
