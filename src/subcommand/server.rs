use super::*;

use std::io::Read;

use io::BufReader;

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
          let entry = entry.unwrap();

          if entry.file_type().is_dir() {
            continue;
          }

          let path =
            Utf8Path::from_path(entry.path()).context(error::PathUnicode { path: entry.path() })?;

          let mut reader = BufReader::new(File::open(path).context(error::FilesystemIo { path })?);

          let mut bytes = [0u8; MAGIC_BYTES.len()];
          reader.read_exact(&mut bytes).unwrap();

          if bytes != MAGIC_BYTES {
            continue;
          }

          let mut manifest_hash = [0u8; Hash::LEN];
          reader.read_exact(&mut manifest_hash).unwrap();

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
