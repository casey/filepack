use {
  super::*,
  axum::{
    Router,
    extract::{Extension, Path},
    http::header,
    routing::{get, put},
  },
  axum_server::Handle,
  std::net::TcpStream,
  tokio::{net::TcpListener, runtime},
  tokio_util::io::ReaderStream,
};

static THREAD_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Parser)]
pub(crate) struct Serve {
  #[arg(help = "Listen on <ADDRESS> for incoming requests", long)]
  address: String,
  #[arg(
    help = "Write listening port to <ADDRESS>",
    long,
    value_name = "ADDRESS"
  )]
  ready_address: Option<SocketAddr>,
}

impl Serve {
  async fn download(
    server: Extension<Arc<Server>>,
    Path(hash): Path<Hash>,
  ) -> ServerResult<Response> {
    let (file, len) = server.open_file(hash).await?;

    Ok(
      Response::builder()
        .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
        .header(header::CONTENT_LENGTH, len)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::ETAG, format!("\"{hash}\""))
        .body(Body::from_stream(ReaderStream::new(file)))
        .unwrap(),
    )
  }

  pub(crate) fn router(server: Arc<Server>) -> Router {
    Router::new()
      .route("/{hash}", get(Self::download))
      .route("/{hash}", put(Self::upload))
      .layer(Extension(server))
  }

  pub(crate) fn run(self, options: Options) -> Result {
    let runtime = runtime::Builder::new_multi_thread()
      .name("server")
      .thread_name_fn(|| {
        format!(
          "server-thread-{}",
          THREAD_COUNTER.fetch_add(1, atomic::Ordering::Relaxed)
        )
      })
      .enable_all()
      .build()
      .context(error::ServerRuntime)?;

    runtime.block_on(self.serve(options))?;

    Ok(())
  }

  async fn serve(self, options: Options) -> Result {
    let handle = Handle::new();

    {
      let handle = handle.clone();
      let mut shutting_down = false;
      ctrlc::set_handler(move || {
        if shutting_down {
          process::exit(1);
        }

        handle.graceful_shutdown(Some(Duration::from_millis(100)));

        shutting_down = true;
      })
      .unwrap();
    }

    let server = Arc::new(Server::new(options)?);

    let router = Self::router(server);

    let listener = TcpListener::bind(&self.address)
      .await
      .context(error::BindListener {
        address: self.address,
      })?
      .into_std()
      .context(error::ListenerIntoStandard)?;

    if let Some(address) = self.ready_address {
      let port = listener.local_addr().context(error::LocalAddress)?.port();

      let mut stream = TcpStream::connect(address).context(error::ReadyAddress { address })?;

      stream
        .write_all(port.to_string().as_bytes())
        .context(error::ReadyAddress { address })?;
    }

    axum_server::from_tcp(listener)
      .context(error::Serve)?
      .handle(handle)
      .serve(router.into_make_service())
      .await
      .context(error::Serve)?;

    Ok(())
  }

  async fn upload(server: Extension<Arc<Server>>, hash: Path<Hash>, body: Body) -> ServerResult {
    server.write_file(*hash, body).await
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    axum::{body::to_bytes, http::Request},
    tower::ServiceExt,
  };

  struct TestServer {
    data_dir: Utf8PathBuf,
    router: Router,
    #[allow(unused)]
    tempdir: TempDir,
  }

  impl TestServer {
    async fn get(&self, path: &str) -> Response {
      self
        .router
        .clone()
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap()
    }

    fn new() -> Self {
      let (tempdir, data_dir) = tempdir();

      let server = Arc::new(Server::with_data_dir(&data_dir).unwrap());

      let router = Serve::router(server);

      Self {
        data_dir,
        router,
        tempdir,
      }
    }

    async fn put(&self, path: &str, content: &[u8]) -> Response {
      self
        .router
        .clone()
        .oneshot(
          Request::builder()
            .method("PUT")
            .uri(path)
            .body(Body::from(content.to_vec()))
            .unwrap(),
        )
        .await
        .unwrap()
    }

    fn write_file(&self, hash: Hash, content: &[u8]) {
      fs::write(self.data_dir.join("files").join(hash.to_string()), content).unwrap();
    }
  }

  #[tokio::test]
  async fn download_response() {
    let server = TestServer::new();

    let hash = Hash::bytes(b"bar");
    server.write_file(hash, b"bar");

    let response = server.get(&format!("/{hash}")).await;

    assert_eq!(response.status(), StatusCode::OK);

    let headers = response.headers();

    assert_eq!(
      headers[header::CACHE_CONTROL],
      "public, max-age=31536000, immutable",
    );
    assert_eq!(headers[header::CONTENT_LENGTH], "3");
    assert_eq!(headers[header::CONTENT_TYPE], "application/octet-stream");
    assert_eq!(headers[header::ETAG], format!("\"{hash}\""));

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&body[..], b"bar");
  }

  #[tokio::test]
  async fn upload_creates_file() {
    let server = TestServer::new();

    let hash = Hash::bytes(b"bar");

    let response = server.put(&format!("/{hash}"), b"bar").await;

    assert_eq!(response.status(), StatusCode::OK);

    assert_eq!(
      fs::read(server.data_dir.join("files").join(hash.to_string())).unwrap(),
      b"bar",
    );

    assert_eq!(
      fs::read_dir(server.data_dir.join("incoming"))
        .unwrap()
        .count(),
      0,
    );
  }

  #[tokio::test]
  async fn upload_short_circuits_when_file_exists() {
    let server = TestServer::new();

    let hash = Hash::bytes(b"bar");

    server.write_file(hash, b"bar");

    let response = server.put(&format!("/{hash}"), b"bar").await;

    assert_eq!(response.status(), StatusCode::OK);

    assert_eq!(
      fs::read(server.data_dir.join("files").join(hash.to_string())).unwrap(),
      b"bar",
    );

    assert_eq!(
      fs::read_dir(server.data_dir.join("incoming"))
        .unwrap()
        .count(),
      0,
    );
  }

  #[tokio::test]
  async fn upload_with_wrong_hash_fails() {
    let server = TestServer::new();

    let actual = Hash::bytes(b"bar");
    let expected = Hash::bytes(b"baz");

    let response = server.put(&format!("/{expected}"), b"bar").await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();

    assert_eq!(
      str::from_utf8(&body).unwrap(),
      format!("expected upload with hash {expected} but got {actual}"),
    );

    assert_eq!(
      fs::read_dir(server.data_dir.join("incoming"))
        .unwrap()
        .count(),
      0,
    );
  }
}
