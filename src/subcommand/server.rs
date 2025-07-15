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
  archives: Vec<Archive>,
}

struct State {
  archives: Vec<Archive>,
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

          if path.extension() != Some(Archive::EXTENSION) {
            continue;
          }

          archives.push(Archive::load(path).context(error::ArchiveLoad { path })?);
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
