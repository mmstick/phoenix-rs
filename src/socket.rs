use crate::chan::Channel;
use crate::event::{Event, PhoenixEvent};
use crate::message::Message as PhoenixMessage;
use async_io::Timer;
use async_native_tls::TlsConnector;
use futures::StreamExt;
use serde_json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time;
use tungstenite::Message;

pub struct Phoenix {
  count: u8,
  channels: Arc<Mutex<Vec<Arc<Mutex<Channel>>>>>,
  sender: flume::Sender<Message>,
}

impl Phoenix {
  pub fn new(
    sender: flume::Sender<Message>,
    receiver: flume::Receiver<Message>,
    callback: flume::Sender<PhoenixMessage>,
    url: &str,
  ) -> Phoenix {
    Phoenix::new_with_parameters(sender, receiver, callback, url, &HashMap::new())
  }

  pub fn new_with_parameters(
    sender: flume::Sender<Message>,
    receiver: flume::Receiver<Message>,
    callback: flume::Sender<PhoenixMessage>,
    url: &str,
    params: &HashMap<&str, &str>,
  ) -> Phoenix {
    let full_url = if params.is_empty() {
      format!("{}/websocket", url)
    } else {
      let mut joined_params = "".to_owned();
      for (index, (key, value)) in params.iter().enumerate() {
        joined_params += if index == 0 { "?" } else { "&" };
        joined_params += key;
        joined_params += "=";
        joined_params += value;
      }
      format!("{}/websocket{}", url, joined_params)
    };

    debug!("connect socket to URL: {}", full_url);

    let copy_callback = callback.clone();

    let connection = async move {
      let (tx, mut rx) = match crate::websocket::connect(&full_url, TlsConnector::new()).await {
        Ok((stream, _)) => stream.split(),
        Err(why) => {
          let _ = copy_callback.send(PhoenixMessage {
            topic: "phoenix".to_string(),
            event: Event::Defined(PhoenixEvent::Close),
            reference: None,
            join_ref: None,
            payload: serde_json::Value::Null,
          });

          error!("{:?}", why);

          return Err(why);
        }
      };

      // Receives and responds to Messages received through the websocket
      let socket_stream = {
        let callback = copy_callback.clone();
        stream! {
          while let Some(message) = rx.next().await {
            let message = match message {
              Ok(message) => message,
              Err(why) => {
                let _ = callback.send(PhoenixMessage {
                  topic: "phoenix".to_string(),
                  event: Event::Defined(PhoenixEvent::Close),
                  reference: None,
                  join_ref: None,
                  payload: serde_json::Value::Null,
                });

                error!("{:?}", why);

                break;
              }
            };

            match message {
              Message::Text(message) => {
                let message: PhoenixMessage = serde_json::from_str(&message).unwrap();
                let _ = callback.send(message);
              }

              Message::Ping(msg) => yield Message::Pong(msg),

              Message::Close(msg) => {
                let _ = callback.send(PhoenixMessage {
                  topic: "phoenix".to_string(),
                  event: Event::Defined(PhoenixEvent::Close),
                  reference: None,
                  join_ref: None,
                  payload: serde_json::Value::Null,
                });

                yield Message::Close(msg);
              }

              _ => (),
            }
          }
        }
      };

      if let Err(why) = futures::stream::select(socket_stream, receiver.into_stream())
        .map(Ok)
        .forward(tx)
        .await
      {
        let _ = copy_callback.send(PhoenixMessage {
          topic: "phoenix".to_string(),
          event: Event::Defined(PhoenixEvent::Close),
          reference: None,
          join_ref: None,
          payload: serde_json::Value::Null,
        });
        error!("{:?}", why);
      }

      Ok(())
    };

    let tx = sender.clone();
    let heartbeat = async move {
      loop {
        let stdin_sink = tx.clone();

        let msg = PhoenixMessage {
          topic: "phoenix".to_owned(),
          event: Event::Defined(PhoenixEvent::Heartbeat),
          reference: None,
          join_ref: None,
          payload: serde_json::from_str("{}").unwrap(),
        };

        let message = Message::Text(serde_json::to_string(&msg).unwrap());

        if stdin_sink.send(message).is_err() {
          error!("unable to send Heartbeat");
          break;
        }

        Timer::after(time::Duration::from_secs(30)).await;
      }
    };

    smol::spawn(futures::future::join(connection, heartbeat)).detach();

    let channels: Arc<Mutex<Vec<Arc<Mutex<Channel>>>>> = Arc::new(Mutex::new(vec![]));

    Phoenix {
      count: 0,
      channels,
      sender,
    }
  }

  pub fn channel(&mut self, topic: &str) -> Arc<Mutex<Channel>> {
    self.count += 1;
    let chan = Arc::new(Mutex::new(Channel::new(
      topic,
      self.sender.clone(),
      &format!("{}", self.count),
    )));
    let mut channels = self.channels.lock().unwrap();
    channels.push(chan.clone());
    chan
  }
}
