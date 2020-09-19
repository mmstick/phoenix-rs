use serde_json::Value;
// use websocket::Message;

// use websocket::futures::sync::mpsc::Sender;
// use websocket::futures::Sink;

use crate::event::{Event, PhoenixEvent};
use crate::message::Message as PhoenixMessage;
use tungstenite::Message;

pub struct Channel {
  topic: String,
  reference: String,
  sender: flume::Sender<Message>,
}

impl Channel {
  pub fn new(topic: &str, sender: flume::Sender<Message>, reference: &str) -> Channel {
    Channel {
      topic: topic.to_owned(),
      reference: reference.to_owned(),
      sender,
    }
  }

  pub fn send_message(&mut self, message: Message) {
    if let Err(msg) = self.sender.send(message) {
      error!("{:?}", msg)
    }
  }

  fn build_message(&mut self, event: Event, payload: Value) -> Message {
    let message = PhoenixMessage {
      topic: self.topic.to_owned(),
      event,
      reference: Some(self.reference.to_owned()),
      join_ref: Some(self.reference.to_owned()),
      payload,
    };

    Message::Text(serde_json::to_string(&message).unwrap())
  }

  pub fn send(&mut self, event: Event, msg: &Value) {
    let message = self.build_message(event, msg.to_owned());
    self.send_message(message);
  }

  pub fn join(&mut self) {
    let message = self.build_message(Event::Defined(PhoenixEvent::Join), Value::Null);
    self.send_message(message);
  }

  pub fn join_with_message(&mut self, payload: Value) {
    let message = self.build_message(Event::Defined(PhoenixEvent::Join), payload);
    self.send_message(message);
  }
}
