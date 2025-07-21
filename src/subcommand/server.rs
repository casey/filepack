use {
  self::templates::{IndexHtml, PackageHtml, PageContent, PageHtml},
  super::*,
  axum::{extract::Path, routing::get, Extension, Router},
  server_error::ServerError,
};

// todo:
// - test package.html rendering

mod templates;

#[derive(Parser)]
pub(crate) struct Server {
  #[arg(
    default_value = "0.0.0.0",
    help = "Listen on <ADDRESS> for incoming requests.",
    long
  )]
  address: String,
  #[arg(help = "Serve packages from directory <PACKAGES>.")]
  packages: Utf8PathBuf,
  #[arg(
    default_value_t = 80,
    help = "Listen on <PORT> for incoming requests.",
    long
  )]
  port: u16,
}

struct State {
  archives: Vec<Package>,
}

impl Server {
  async fn index(state: Extension<Arc<State>>) -> PageHtml<IndexHtml> {
    IndexHtml {
      packages: state.archives.clone(),
    }
    .page()
  }

  fn load(&self) -> Result<Router> {
    let mut archives = Vec::new();

    for entry in WalkDir::new(&self.packages) {
      let entry = entry?;

      if entry.file_type().is_dir() {
        continue;
      }

      let path = decode_path(entry.path())?;

      if path.file_name() != Some("filepack.json") {
        continue;
      }

      archives.push(Package::load(path)?);
    }

    archives.sort_by_key(|archive| archive.hash);

    Ok(
      Router::new()
        .route("/", get(Self::index))
        .route("/package/{hash}", get(Self::package))
        .layer(Extension(Arc::new(State { archives }))),
    )
  }

  async fn package(
    state: Extension<Arc<State>>,
    Path(hash): Path<Hash>,
  ) -> Result<PageHtml<PackageHtml>, ServerError> {
    Ok(
      PackageHtml {
        package: state
          .archives
          .iter()
          .find(|archive| archive.hash == hash)
          .cloned()
          .ok_or_else(|| ServerError::NotFound(format!("package `{hash}` not found")))?,
      }
      .page(),
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
  use {super::*, axum_test::TestServer, reqwest::StatusCode};

  fn server(path: impl Into<OsString>) -> TestServer {
    TestServer::new(
      Server::try_parse_from(["filepack".into(), path.into()])
        .unwrap()
        .load()
        .unwrap(),
    )
    .unwrap()
  }

  #[tokio::test]
  async fn index_lists_archives() {
    let dir = TempDir::new().unwrap();

    dir.child("foo").create_dir_all().unwrap();
    dir.child("foo/hello.txt").write_str("hello").unwrap();

    let path = Utf8Path::from_path(dir.path()).unwrap();

    command!("create", path.join("foo"));

    dir.child("bar").create_dir_all().unwrap();
    dir.child("foo/hello.txt").write_str("hello").unwrap();

    command!("create", path.join("bar"));

    let foo = Package::load(&path.join("foo/filepack.json")).unwrap();
    let bar = Package::load(&path.join("bar/filepack.json")).unwrap();

    let server = server(dir.path());

    let response = server.get("/").await;

    response.assert_status_ok();

    response.assert_text(
      IndexHtml {
        packages: vec![foo, bar],
      }
      .page()
      .to_string(),
    );
  }

  #[tokio::test]
  async fn package_endpoint_returns_archive_details() {
    let dir = TempDir::new().unwrap();

    dir.child("foo").create_dir_all().unwrap();
    dir.child("foo/hello.txt").write_str("hello").unwrap();

    let path = Utf8Path::from_path(dir.path()).unwrap();

    command!("create", path.join("foo"));

    let package = Package::load(&path.join("foo/filepack.json")).unwrap();

    let server = server(dir.path());

    let response = server.get(&format!("/package/{}", package.hash)).await;

    response.assert_status_ok();

    response.assert_text(PackageHtml { package }.page().to_string());
  }

  #[tokio::test]
  async fn package_endpoint_panics_for_nonexistent_archive() {
    let dir = TempDir::new().unwrap();

    let server = server(dir.path());

    let response = server
      .get("/package/0000000000000000000000000000000000000000000000000000000000000000")
      .await;

    response.assert_status(StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  async fn package_endpoint_handles_multiple_archives() {
    let dir = TempDir::new().unwrap();

    dir.child("foo").create_dir_all().unwrap();
    dir.child("foo/hello.txt").write_str("hello").unwrap();

    let path = Utf8Path::from_path(dir.path()).unwrap();

    command!("create", path.join("foo"));

    dir.child("bar").create_dir_all().unwrap();
    dir.child("foo/hello.txt").write_str("hello").unwrap();

    command!("create", path.join("bar"));

    // let foo = Package::load(&path.join("foo.filepack")).unwrap();

    // let bar = Package::load(&path.join("bar.filepack")).unwrap();

    // assert_ne!(foo.hash, bar.hash);

    // let server = server(dir.path());

    // let response = server.get(&format!("/package/{}", foo.hash)).await;
    // response.assert_status_ok();
    // response.assert_text(PackageHtml { archive: foo }.page().to_string());

    // let response = server.get(&format!("/package/{}", bar.hash)).await;
    // response.assert_status_ok();
    // response.assert_text(PackageHtml { archive: bar }.page().to_string());
  }
}
