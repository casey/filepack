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

  fn load(&self) -> Result<Router> {
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

    archives.sort_by_key(|archive| archive.manifest);

    Ok(
      Router::new()
        .route("/", get(Self::index))
        .layer(Extension(Arc::new(State { archives }))),
    )
  }

  pub(crate) fn run(self) -> Result {
    Runtime::new()
      .context(error::ServerRuntime)?
      .block_on(async {
        let app = Self::load(&self)?;

        let listener = tokio::net::TcpListener::bind((self.address.as_str(), self.port))
          .await
          .context(error::ServerBind {
            address: self.address,
            port: self.port,
          })?;

        axum::serve(listener, app).await.context(error::ServerRun)?;

        Ok(())
      })
  }
}

#[cfg(test)]
mod tests {
  use {super::*, axum_test::TestServer, std::iter};

  fn test_archive(n: u8) -> Vec<u8> {
    b"FILEPACK"
      .iter()
      .copied()
      .chain(iter::repeat_n(n, 32))
      .collect()
  }

  #[tokio::test]
  async fn index_lists_archives() {
    let dir = TempDir::new().unwrap();

    dir
      .child("foo.filepack")
      .write_binary(&test_archive(0x00))
      .unwrap();

    dir
      .child("bar.filepack")
      .write_binary(&test_archive(0xFF))
      .unwrap();

    let server = match Server::try_parse_from(["filepack", dir.path().to_str().unwrap()]) {
      Ok(server) => server,
      Err(error) => {
        panic!("{}", error.to_string());
      }
    };

    let server = TestServer::new(server.load().unwrap()).unwrap();

    let response = server.get("/").await;

    response.assert_status_ok();

    response.assert_text_contains(
      "\
    <ul>
      <li class=monospace>0000000000000000000000000000000000000000000000000000000000000000</li>
      <li class=monospace>ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff</li>
    </ul>
",
    );
  }
}
