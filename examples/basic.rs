use phoenix::{Event, Phoenix};
use smol::Timer;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
  env_logger::init();

  let url = "ws://localhost:9000/socket";

  let (sender, emitter) = flume::bounded(0);
  let (callback, messages) = flume::bounded(0);

  // Simulate a user
  let simulated_user = async move {
    let mut phx = Phoenix::new(sender, emitter, callback, url);
    let mutex_chan = phx.channel("room:lobby").clone();

    {
      let mut device_chan = mutex_chan.lock().unwrap();
      device_chan.join();
    }

    while let Ok(message) = messages.recv_async().await {
      println!("user1: ${:?}", message);
    }

    Ok(())
  };

  // Simulate an other user

  let other_user = async move {
    Timer::after(Duration::from_millis(500)).await;

    let (sender, emitter) = flume::bounded(0);
    let (callback, messages) = flume::bounded(0);

    let mut phx = Phoenix::new(sender, emitter, callback, url);
    let mutex_chan = phx.channel("room:lobby").clone();

    {
      let mut device_chan = mutex_chan.lock().unwrap();
      device_chan.join();
      let body = serde_json::from_str(r#"{"body": "Hello"}"#).unwrap();
      device_chan.send(Event::Custom("new_msg".to_string()), &body);
    }

    while let Ok(message) = messages.recv_async().await {
      println!("user2: {:?}", message);
    }

    Ok(())
  };

  let (a, b) = smol::block_on(futures::future::join(simulated_user, other_user));
  a.and(b)
}
