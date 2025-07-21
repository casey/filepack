use {
  self::{
    server_error::ServerError,
    templates::{IndexHtml, PackageHtml, PageContent, PageHtml},
  },
  super::*,
  axum::{extract::Path, routing::get, Extension, Router},
};

mod server_error;
mod templates;

#[derive(Parser)]
pub(crate) struct Server {
  #[arg(
    default_value = "0.0.0.0",
    help = "Listen on <ADDRESS> for incoming requests.",
    long
  )]
  address: String,
  #[arg(help = "Serve packages from <PACKAGES> directory.")]
  packages: Utf8PathBuf,
  #[arg(
    default_value_t = 80,
    help = "Listen on <PORT> for incoming requests.",
    long
  )]
  port: u16,
}

struct State {
  packages: BTreeMap<Hash, Package>,
}

impl Server {
  async fn index(state: Extension<Arc<State>>) -> PageHtml<IndexHtml> {
    IndexHtml {
      packages: state.packages.clone(),
    }
    .page()
  }

  fn load(&self) -> Result<Router> {
    let mut packages = BTreeMap::new();

    for entry in WalkDir::new(&self.packages) {
      let entry = entry?;

      if entry.file_type().is_dir() {
        continue;
      }

      let path = decode_path(entry.path())?;

      if path.file_name() != Some(Manifest::FILENAME) {
        continue;
      }

      let package = Package::load(path)?;

      packages.insert(package.fingerprint, package);
    }

    Ok(
      Router::new()
        .route("/", get(Self::index))
        .route("/package/{hash}", get(Self::package))
        .layer(Extension(Arc::new(State { packages }))),
    )
  }

  async fn package(
    state: Extension<Arc<State>>,
    Path(hash): Path<Hash>,
  ) -> Result<PageHtml<PackageHtml>, ServerError> {
    Ok(
      PackageHtml {
        package: state
          .packages
          .get(&hash)
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
  async fn index_lists_packages() {
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
        packages: [(foo.fingerprint, foo), (bar.fingerprint, bar)].into(),
      }
      .page()
      .to_string(),
    );
  }

  #[tokio::test]
  async fn package_endpoint_returns_package_details() {
    let dir = TempDir::new().unwrap();

    let path = Utf8Path::from_path(dir.path()).unwrap();

    dir.child("foo").create_dir_all().unwrap();
    dir.child("foo/foo.txt").write_str("foo").unwrap();
    dir
      .child("foo/metadata.json")
      .write_str(r#"{ "title": "foo" }"#)
      .unwrap();

    command!("create", path.join("foo"));

    dir.child("bar").create_dir_all().unwrap();
    dir.child("bar/bar.txt").write_str("bar").unwrap();
    dir
      .child("bar/metadata.json")
      .write_str(r#"{ "title": "bar" }"#)
      .unwrap();

    command!("create", path.join("bar"));

    let server = server(dir.path());

    let package = Package::load(&path.join("foo/filepack.json")).unwrap();

    assert!(package.metadata.is_some());

    let response = server
      .get(&format!("/package/{}", package.fingerprint))
      .await;

    response.assert_status_ok();

    response.assert_text(PackageHtml { package }.page().to_string());

    let package = Package::load(&path.join("bar/filepack.json")).unwrap();

    let response = server
      .get(&format!("/package/{}", package.fingerprint))
      .await;

    response.assert_status_ok();

    response.assert_text(PackageHtml { package }.page().to_string());
  }

  #[tokio::test]
  async fn package_endpoint_returns_not_found_for_nonexistent_package() {
    let dir = TempDir::new().unwrap();

    let server = server(dir.path());

    let response = server
      .get("/package/0000000000000000000000000000000000000000000000000000000000000000")
      .await;

    response.assert_status(StatusCode::NOT_FOUND);
  }
}
