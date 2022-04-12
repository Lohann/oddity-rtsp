mod server;
mod media;
mod settings;

use std::error::Error;
use std::env::args;
use std::path::Path;

use settings::{Settings, MediaKind};
use media::{MediaController, Source, Multiplexer};
use server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_env("LOG"))
    .pretty()
    .init();

  let settings_file = args()
    .nth(1)
    .unwrap_or("default.yaml".to_string());
  let settings_file = Path::new(&settings_file);

  let settings = Settings::from_file(settings_file)?;
  tracing::debug!(?settings, "read settings file");

  let mut media_controller = MediaController::new();
  for media_item in settings.media.iter() {
    let source = match media_item.kind {
      MediaKind::Multiplex =>
        Source::Multiplex(Multiplexer::new(media_item.uri.parse()?)),
    };

    media_controller.register_source(&media_item.path, source);
  }

  tracing::info!(%media_controller, "initialized media controller");

  let server = Server::new(("localhost", 5554));
  server.run().await
}