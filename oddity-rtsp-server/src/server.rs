use std::net::{
  ToSocketAddrs,
  TcpListener,
};
use std::error::Error;
use std::sync::{Arc, Mutex};

use concurrency::ServicePool;

use super::{
  media,
  connection::Connection,
};

// TODO duplicate
type MediaController = Arc<Mutex<media::Controller>>;

pub struct Server<A: ToSocketAddrs + 'static> {
  addrs: A,
  media: MediaController,
  connections: ServicePool,
}

impl<A: ToSocketAddrs + 'static> Server<A> {

  pub fn new(
    addrs: A,
    media: media::Controller,
  ) -> Self {
    Self {
      addrs,
      media: Arc::new(
        Mutex::new(
          media
        )
      ),
      connections: ServicePool::new(),
    }
  }

  pub fn run(
    self
  ) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&self.addrs)?;
    loop {
      let (socket, addr) = listener.accept()?;
      tracing::trace!(%addr, "accepted client");

      self.connections.spawn(
        move |stop_rx| {
          Connection::new(
              socket,
              &self.media,
              stop_rx,
            )
            .run();
        }
      );
    }
  }

}
