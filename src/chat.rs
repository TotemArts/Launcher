use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use futures::Stream;
use futures::Sink;
use futures::Future;

pub struct Chat {
  connected: Arc<Mutex<bool>>,
  rx: Receiver<ChatEvent>,
  tx: futures::sync::mpsc::Sender<ChatEvent>,
  join_handle: std::thread::JoinHandle<Result<(), std::io::Error>>,
}

pub enum ChatEvent {
  Config(Config),
  Send(String),
  Receive(Message),
  Error(Error),
  Username(String),
  Connect,
  Disconnect,
}

pub struct Message {
  //add something like a timestamp?
  pub username: String,
  pub message: String,
}

struct Error {
  description: String,
}

struct Config {
  username: String,
}

impl Chat {
  pub fn new() -> Chat {
    let (client_sender, mut server_receiver) = futures::sync::mpsc::channel(10);
    let (server_sender, client_receiver) = channel();
    let connected = Arc::new(Mutex::new(false));
    Chat {
      connected: connected.clone(),
      rx: client_receiver,
      tx: client_sender,
      join_handle: thread::spawn(move || {
        let mut config_option : Option<Config> = None;
        
        loop {

          let recv = match server_receiver.poll().unwrap() {
            futures::Async::Ready(value) => value,
            _ => None,
          };
          if let Some(chat_event) = recv {
            match chat_event {
              ChatEvent::Config(new_config) => {
                config_option = Some(new_config);
              },
              ChatEvent::Connect => {
                if let Some(mut config) = config_option.as_mut() {
                  let mut rt = tokio::runtime::current_thread::Runtime::new().unwrap();
                  let client = tokio_xmpp::Client::new("Sonny@weird-server.com", "").unwrap();
                  let (sink, stream) = client.split();
                  rt.spawn(
                    server_receiver.for_each(|chat_event| {
                      Ok(())
                    }).map_err(|e| panic!("Potatoes"))
                  );
                } else {
                  server_sender.send(ChatEvent::Error(Error{description: "".to_string()}));
                }
              },
              _ => {},
            }
          };
        }
        Ok(())
      }),
    }
  }

  pub fn send(&self, message: String) {
    self.tx.clone().send(ChatEvent::Send(message)).wait();
  }

  pub fn receive(&self) -> Option<Message> {
    let receive = self.rx.recv();
    match receive {
      Ok(chat_event) => {
        match chat_event {
          ChatEvent::Receive(message) => Some(message),
          _ => { None }
        }
      },
      Err(e) => {
        None
      }
    }
  }

  pub fn is_connected(&self) -> bool {
    *self.connected.lock().unwrap()
  }

  pub fn connect(&self) {
    self.tx.clone().send(ChatEvent::Connect).wait();
  }

  pub fn disconnect(&self){
    self.tx.clone().send(ChatEvent::Disconnect).wait();
  }

  pub fn set_username(&self, username: String) {
    self.tx.clone().send(ChatEvent::Username(encode(username))).wait();
  }
}

pub fn encode(input: String) -> String {
  let mut output = String::new();
  for chr in input.chars() {
    match chr {
      '0'..='9' | 'a'..='z' | 'A'..='Z' => {
        output.push(chr);
      },
      _ => {
        output.push_str(&format!("#{};", chr as u32));
      },
    }
  }
  output
}
