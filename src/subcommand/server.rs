use {
  super::*,
  axum::{extract::Path, routing::get, Extension, Router},
  server_error::ServerError,
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

#[derive(Boilerplate)]
struct PackageHtml {
  archive: Archive,
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

  async fn package(
    state: Extension<Arc<State>>,
    Path(hash): Path<Hash>,
  ) -> Result<PackageHtml, ServerError> {
    Ok(PackageHtml {
      archive: state
        .archives
        .iter()
        .find(|archive| archive.manifest_hash == hash)
        .cloned()
        .ok_or_else(|| ServerError::NotFound(format!("package `{hash}` not found")))?,
    })
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

    archives.sort_by_key(|archive| archive.manifest_hash);

    Ok(
      Router::new()
        .route("/", get(Self::index))
        .route("/package/{hash}", get(Self::package))
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
  use {super::*, axum_test::TestServer, reqwest::StatusCode, std::iter};

  fn test_archive(n: u8) -> Vec<u8> {
    b"FILEPACK"
      .iter()
      .copied()
      .chain(iter::repeat_n(n, 32))
      .collect()
  }

  fn server(dir_path: &str) -> TestServer {
    let server = Server::try_parse_from(["filepack", dir_path]).unwrap();
    TestServer::new(server.load().unwrap()).unwrap()
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

    let server = server(dir.path().to_str().unwrap());

    let response = server.get("/").await;

    response.assert_status_ok();

    // response.assert_text(
    //   IndexHtml {
    //     archives: vec![
    //       Archive {
    //         manifest_hash: [0x00; 32].into(),
    //       },
    //       Archive {
    //         manifest_hash: [0xFF; 32].into(),
    //       },
    //     ],
    //   }
    //   .to_string(),
    // );
  }

  #[tokio::test]
  async fn package_endpoint_returns_archive_details() {
    let dir = TempDir::new().unwrap();

    dir
      .child("test.filepack")
      .write_binary(&test_archive(0xAB))
      .unwrap();

    let server = server(dir.path().to_str().unwrap());

    let response = server
      .get("/package/abababababababababababababababababababababababababababababababab")
      .await;

    response.assert_status_ok();

    // response.assert_text(
    //   PackageHtml {
    //     archive: Archive {
    //       manifest_hash: [0xAB; 32].into(),
    //     },
    //   }
    //   .to_string(),
    // );
  }

  #[tokio::test]
  async fn package_endpoint_panics_for_nonexistent_archive() {
    let dir = TempDir::new().unwrap();

    dir
      .child("test.filepack")
      .write_binary(&test_archive(0x00))
      .unwrap();

    let server = server(dir.path().to_str().unwrap());

    let response = server
      .get("/package/deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef")
      .await;

    response.assert_status(StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  async fn package_endpoint_handles_multiple_archives() {
    let dir = TempDir::new().unwrap();

    dir
      .child("first.filepack")
      .write_binary(&test_archive(0x11))
      .unwrap();

    dir
      .child("second.filepack")
      .write_binary(&test_archive(0x22))
      .unwrap();

    let server = server(dir.path().to_str().unwrap());

    let response = server
      .get("/package/1111111111111111111111111111111111111111111111111111111111111111")
      .await;

    response.assert_status_ok();

    // response.assert_text(
    //   PackageHtml {
    //     archive: Archive {
    //       manifest_hash: [0x11; 32].into(),
    //     },
    //   }
    //   .to_string(),
    // );

    let response = server
      .get("/package/2222222222222222222222222222222222222222222222222222222222222222")
      .await;

    response.assert_status_ok();

    // response.assert_text(
    //   PackageHtml {
    //     archive: Archive {
    //       manifest_hash: [0x22; 32].into(),
    //     },
    //   }
    //   .to_string(),
    // );
  }
}
