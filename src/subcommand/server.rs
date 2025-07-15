use super::*;

#[derive(Parser)]
pub(crate) struct Server {
  #[arg(help = "Archive directory containing filepack archives.")]
  archives: Utf8PathBuf,
  #[arg(help = "Port to run the server on.", default_value_t = 80)]
  port: u16,
}

use boilerplate::Boilerplate;

#[derive(Boilerplate)]
struct IndexHtml {
  archives: Vec<Hash>,
}

impl Server {
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
              return Err(error::ArchiveEof { path }.into_error(error));
            }
            Err(error) => {
              return Err(error::FilesystemIo { path }.into_error(error));
            }
          }

          archives.push(Hash::from(manifest_hash));
        }

        let app = Router::new().route("/", get(|| async { IndexHtml { archives } }));

        let listener = tokio::net::TcpListener::bind(("0.0.0.0", self.port))
          .await
          .context(error::ServerBind { port: self.port })?;

        axum::serve(listener, app)
          .await
          .context(error::ServerServe)?;

        Ok(())
      })
  }
}
