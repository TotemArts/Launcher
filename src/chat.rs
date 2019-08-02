use std::thread;
use std::sync::{Arc, Mutex};
use futures::unsync::mpsc::{unbounded, UnboundedSender, UnboundedReceiver};
use tokio_xmpp::{Client, Packet};
use xmpp_parsers::{Jid, Element, TryFrom};
use xmpp_parsers::message::{Body, MessageType};
use xmpp_parsers::presence::{Presence, Show as PresenceShow, Type as PresenceType};


//traits:
use futures::{Stream, Sink, Future};

/// The generic chat instance, should be transformed into a trait
pub struct Chat<R: FnMut(ChatEvent)> {
  connected: Arc<Mutex<bool>>,
  receiver: R,
  sender: UnboundedSender<tokio_xmpp::xmpp_codec::Packet>,
  rt: tokio::runtime::Runtime,
}

pub enum ChatEvent {
/// Send a string
  Send(String),
/// Receive a message
  Receive(Message),
/// Any event, like connect, disconnect, list of users?, etc.
  Event(String),
/// Any error that occurs
  Error(Error)
}

pub struct Message {
  //add something like a timestamp?
  pub username: String,
  pub message: String,
}

pub struct Error {
  pub description: String,
}

fn make_presence() -> Element {
  let mut presence = Presence::new(PresenceType::None);
  presence.show = Some(PresenceShow::Chat);
  presence.statuses.insert(String::from("en"), String::from("Echoing messages."));
  presence.into()
}

impl<R: FnMut(ChatEvent)> Chat<R> {
  pub fn new(receiver: R, username: String, password: String, server: String) -> Chat<R> {
    let (client_sender, mut server_receiver) = unbounded();
    let connected = Arc::new(Mutex::new(false));
    let chat = Chat {
      connected: connected.clone(),
      receiver: receiver,
      sender: client_sender,
      rt: tokio::runtime::Runtime::new().unwrap(),
    };
    let client = tokio_xmpp::Client::new(&format!("{}@{}", username, server), &password).unwrap();
    let (sink, stream) = client.split();
/*
    chat.rt.spawn(
      server_receiver.forward(
        sink.sink_map_err(|_| panic!("Pipe"))
      ).map(|(server_receiver, mut sink)| {
        drop(server_receiver);
        let _ = sink.close();
      }).map_err(|e| {
        panic!("Send error: {:?}", e);
      })
    );
*/
/*
    chat.rt.block_on(stream.for_each(|event| {
      if event.is_online() {
        println!("Online!");
        //let presence = make_presence();
        //tx.start_send(Packet::Stanza(presence)).unwrap();
      } else if let Some(message) = event.into_stanza().and_then(|stanza| xmpp_parsers::message::Message::try_from(stanza).ok()) {
        println!("{:?}", message);
      }
      futures::future::ok(())
    }).map_err(|e| {
      panic!("Send error: {:?}", e);
    }));
*/
    chat
  }
/*
  pub fn send(&self, message: String) {
    self.rt.spawn(self.sender.send(message));
  }
*/
  pub fn is_connected(&self) -> bool {
    *self.connected.lock().unwrap()
  }
}

/// encodes an input to an html-encoded output
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
