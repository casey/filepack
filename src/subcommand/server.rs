use {
  super::*,
  axum::{routing::get, Extension, Router},
};

#[derive(Parser)]
pub(crate) struct Server {
  #[arg(
    default_value = "0.0.0.0",
    help = "Listen on <ADDRESS> for incoming requests.",
    long
  )]
  address: String,
  #[arg(help = "Serve archives from directory <ARCHIVES>.")]
  archives: Utf8PathBuf,
  #[arg(
    default_value_t = 80,
    help = "Listen on <PORT> for incoming requests.",
    long
  )]
  port: u16,
}

#[derive(Boilerplate)]
struct IndexHtml {
  archives: Vec<Hash>,
}

struct State {
  archives: Vec<Hash>,
}

impl Server {
  async fn index(state: Extension<Arc<State>>) -> IndexHtml {
    IndexHtml {
      archives: state.archives.clone(),
    }
  }

  pub(crate) fn run(self) -> Result {
    Runtime::new()
      .context(error::ServerRuntime)?
      .block_on(async {
        let mut archives = Vec::new();

        for entry in WalkDir::new(&self.archives) {
          let entry = entry?;

          if entry.file_type().is_dir() {
            continue;
          }

          let path = decode_path(entry.path())?;

          let mut reader = BufReader::new(File::open(path).context(error::FilesystemIo { path })?);

          let mut bytes = [0u8; MAGIC_BYTES.len()];

          match reader.read_exact(&mut bytes) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
              continue;
            }
            Err(error) => {
              return Err(error::FilesystemIo { path }.into_error(error));
            }
          }

          if bytes != MAGIC_BYTES {
            continue;
          }

          let mut manifest_hash = [0u8; Hash::LEN];

          match reader.read_exact(&mut manifest_hash) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => {
              return Err(error::ArchiveTruncated { path }.into_error(error));
            }
            Err(error) => {
              return Err(error::FilesystemIo { path }.into_error(error));
            }
          }

          archives.push(Hash::from(manifest_hash));
        }

        let app = Router::new()
          .route("/", get(Self::index))
          .layer(Extension(Arc::new(State { archives })));

        let listener = tokio::net::TcpListener::bind((self.address.as_str(), self.port))
          .await
          .context(error::ServerBind {
            address: self.address,
            port: self.port,
          })?;

        axum::serve(listener, app)
          .await
          .context(error::ServerServe)?;

        Ok(())
      })
  }
}
