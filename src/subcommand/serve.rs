use {
  super::*,
  axum::{
    Router,
    extract::{Extension, Path},
    http::{Uri, header},
    response::Redirect,
    routing::{get, put},
  },
  axum_server::Handle,
  rustls_acme::{
    AcmeConfig, acme::LETS_ENCRYPT_PRODUCTION_DIRECTORY, axum::AxumAcceptor, caches::DirCache,
  },
  std::net::TcpStream,
  sysinfo::System,
  tokio::{net::TcpListener, runtime},
  tokio_util::io::ReaderStream,
};

static THREAD_COUNTER: AtomicU64 = AtomicU64::new(0);

enum SpawnConfig {
  Http,
  Https(AxumAcceptor),
  Redirect(String),
}

#[derive(Parser)]
pub(crate) struct Serve {
  #[arg(help = "Store ACME TLS certificates in <ACME_CACHE>", long)]
  acme_cache: Option<Utf8PathBuf>,
  #[arg(help = "Provide ACME contact <ACME_CONTACT>", long)]
  acme_contact: Vec<String>,
  #[arg(
    help = "Request ACME TLS certificate for <ACME_DOMAIN>, this server must be reachable at <ACME_DOMAIN>:443 to respond to Let's Encrypt ACME challenges",
    long
  )]
  acme_domain: Vec<String>,
  #[arg(
    default_value = "0.0.0.0",
    help = "Listen on <ADDRESS> for incoming requests",
    long
  )]
  address: String,
  #[arg(help = "Serve HTTP traffic", long)]
  http: bool,
  #[arg(
    help = "Listen on <HTTP_PORT> for incoming HTTP requests [default: 80]",
    long
  )]
  http_port: Option<u16>,
  #[arg(help = "Serve HTTPS traffic", long)]
  https: bool,
  #[arg(
    help = "Listen on <HTTPS_PORT> for incoming HTTPS requests [default: 443]",
    long
  )]
  https_port: Option<u16>,
  #[arg(
    help = "Write listening port to <ADDRESS>",
    long,
    value_name = "ADDRESS"
  )]
  ready_address: Option<SocketAddr>,
  #[arg(help = "Redirect HTTP traffic to HTTPS", long)]
  redirect_http_to_https: bool,
}

impl Serve {
  fn acceptor(&self, acme_cache: Utf8PathBuf) -> Result<AxumAcceptor> {
    static RUSTLS_PROVIDER_INSTALLED: LazyLock<bool> = LazyLock::new(|| {
      rustls::crypto::ring::default_provider()
        .install_default()
        .is_ok()
    });

    ensure!(*RUSTLS_PROVIDER_INSTALLED, error::RustlsProvider);

    let config = AcmeConfig::new(self.acme_domains()?)
      .contact(&self.acme_contact)
      .cache_option(Some(DirCache::new(acme_cache)))
      .directory(LETS_ENCRYPT_PRODUCTION_DIRECTORY);

    let mut state = config.state();

    let mut server_config = rustls::ServerConfig::builder()
      .with_no_client_auth()
      .with_cert_resolver(state.resolver());

    server_config.alpn_protocols = vec!["h2".into(), "http/1.1".into()];

    let acceptor = state.axum_acceptor(Arc::new(server_config));

    tokio::spawn(async move {
      while let Some(result) = state.next().await {
        match result {
          Ok(ok) => eprintln!("ACME event: {ok:?}"),
          Err(err) => eprintln!("ACME error: {err:?}"),
        }
      }
    });

    Ok(acceptor)
  }

  fn acme_domains(&self) -> Result<Vec<String>> {
    if self.acme_domain.is_empty() {
      Ok(vec![System::host_name().context(error::Hostname)?])
    } else {
      Ok(self.acme_domain.clone())
    }
  }

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

  fn http_port(&self) -> Option<u16> {
    if self.redirect_http_to_https
      || self.http
      || self.http_port.is_some()
      || (self.https_port.is_none() && !self.https)
    {
      Some(self.http_port.unwrap_or(80))
    } else {
      None
    }
  }

  fn https_port(&self) -> Option<u16> {
    if self.redirect_http_to_https || self.https || self.https_port.is_some() {
      Some(self.https_port.unwrap_or(443))
    } else {
      None
    }
  }

  fn listener_address(address: &str, port: u16) -> String {
    if address.contains(':') {
      format!("[{address}]:{port}")
    } else {
      format!("{address}:{port}")
    }
  }

  fn redirect_destination(acme_domains: &[String], https_port: u16) -> String {
    if https_port == 443 {
      format!("https://{}", acme_domains[0])
    } else {
      format!("https://{}:{https_port}", acme_domains[0])
    }
  }

  async fn redirect_http_to_https(
    Extension(mut destination): Extension<String>,
    uri: Uri,
  ) -> Redirect {
    if let Some(path_and_query) = uri.path_and_query() {
      destination.push_str(path_and_query.as_str());
    }

    Redirect::to(&destination)
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

    let data_dir = options.data_dir()?;
    let acme_cache = self
      .acme_cache
      .clone()
      .unwrap_or_else(|| data_dir.join("acme-cache"));
    let http_port = self.http_port();
    let https_port = self.https_port();

    let server = Arc::new(Server::with_data_dir(&data_dir)?);

    let router = Self::router(server);

    match (http_port, https_port) {
      (Some(http_port), None) => {
        self
          .spawn(router, handle, http_port, SpawnConfig::Http, true)
          .await?;
      }
      (None, Some(https_port)) => {
        self
          .spawn(
            router,
            handle,
            https_port,
            SpawnConfig::Https(self.acceptor(acme_cache)?),
            true,
          )
          .await?;
      }
      (Some(http_port), Some(https_port)) => {
        let acme_domains = self.acme_domains()?;
        let http_spawn_config = if self.redirect_http_to_https {
          SpawnConfig::Redirect(Self::redirect_destination(&acme_domains, https_port))
        } else {
          SpawnConfig::Http
        };

        tokio::try_join!(
          self.spawn(
            router.clone(),
            handle.clone(),
            http_port,
            http_spawn_config,
            true,
          ),
          self.spawn(
            router,
            handle,
            https_port,
            SpawnConfig::Https(self.acceptor(acme_cache)?),
            false,
          ),
        )?;
      }
      (None, None) => unreachable!(),
    }

    Ok(())
  }

  async fn spawn(
    &self,
    router: Router,
    handle: Handle<SocketAddr>,
    port: u16,
    config: SpawnConfig,
    ready: bool,
  ) -> Result {
    let listener = TcpListener::bind((self.address.as_str(), port))
      .await
      .context(error::BindListener {
        address: Self::listener_address(&self.address, port),
      })?
      .into_std()
      .context(error::ListenerIntoStandard)?;

    if ready && let Some(address) = self.ready_address {
      let port = listener.local_addr().context(error::LocalAddress)?.port();

      let mut stream = TcpStream::connect(address).context(error::ReadyAddress { address })?;

      stream
        .write_all(port.to_string().as_bytes())
        .context(error::ReadyAddress { address })?;
    }

    match config {
      SpawnConfig::Http => {
        axum_server::from_tcp(listener)
          .context(error::Serve)?
          .handle(handle)
          .serve(router.into_make_service())
          .await
          .context(error::Serve)?;
      }
      SpawnConfig::Https(acceptor) => {
        axum_server::from_tcp(listener)
          .context(error::Serve)?
          .handle(handle)
          .acceptor(acceptor)
          .serve(router.into_make_service())
          .await
          .context(error::Serve)?;
      }
      SpawnConfig::Redirect(destination) => {
        axum_server::from_tcp(listener)
          .context(error::Serve)?
          .handle(handle)
          .serve(
            Router::new()
              .fallback(Self::redirect_http_to_https)
              .layer(Extension(destination))
              .into_make_service(),
          )
          .await
          .context(error::Serve)?;
      }
    }

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
    #[track_caller]
    fn assert_incoming_empty(&self) {
      assert_eq!(
        fs::read_dir(self.data_dir.join("incoming"))
          .unwrap()
          .count(),
        0,
      );
    }

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

  #[test]
  fn acme_domain_defaults_to_hostname() {
    assert_eq!(
      serve().acme_domains().unwrap(),
      vec![System::host_name().unwrap()]
    );
  }

  #[test]
  fn acme_domain_flag_is_respected() {
    assert_eq!(
      Serve {
        acme_domain: vec!["foo".into(), "bar".into()],
        ..serve()
      }
      .acme_domains()
      .unwrap(),
      vec!["foo".to_string(), "bar".to_string()],
    );
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

  #[test]
  fn ports() {
    #[track_caller]
    fn case(serve: Serve, http_port: Option<u16>, https_port: Option<u16>) {
      assert_eq!(serve.http_port(), http_port);
      assert_eq!(serve.https_port(), https_port);
    }

    case(serve(), Some(80), None);
    case(
      Serve {
        https: true,
        ..serve()
      },
      None,
      Some(443),
    );
    case(
      Serve {
        https_port: Some(433),
        ..serve()
      },
      None,
      Some(433),
    );
    case(
      Serve {
        http: true,
        https: true,
        ..serve()
      },
      Some(80),
      Some(443),
    );
    case(
      Serve {
        http_port: Some(8080),
        https_port: Some(8443),
        ..serve()
      },
      Some(8080),
      Some(8443),
    );
    case(
      Serve {
        redirect_http_to_https: true,
        ..serve()
      },
      Some(80),
      Some(443),
    );
  }

  #[test]
  fn redirect_destination() {
    let domains = vec!["foo".to_string()];

    assert_eq!(Serve::redirect_destination(&domains, 443), "https://foo");
    assert_eq!(
      Serve::redirect_destination(&domains, 8443),
      "https://foo:8443",
    );
  }

  #[tokio::test]
  async fn redirect_http_to_https() {
    async fn case(path: &str, location: &str) {
      let response = Router::new()
        .fallback(Serve::redirect_http_to_https)
        .layer(Extension("https://foo".to_string()))
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap();

      assert_eq!(response.status(), StatusCode::SEE_OTHER);
      assert_eq!(response.headers()[header::LOCATION], location);
    }

    case("/", "https://foo/").await;
    case("/bar", "https://foo/bar").await;
    case("/bar?baz=qux", "https://foo/bar?baz=qux").await;
  }

  fn serve() -> Serve {
    Serve {
      acme_cache: None,
      acme_contact: Vec::new(),
      acme_domain: Vec::new(),
      address: "0.0.0.0".into(),
      http: false,
      http_port: None,
      https: false,
      https_port: None,
      ready_address: None,
      redirect_http_to_https: false,
    }
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

    server.assert_incoming_empty();
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

    server.assert_incoming_empty();
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

    server.assert_incoming_empty();
  }
}
