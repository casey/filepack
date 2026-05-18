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

pub(crate) struct AuthConfig {
  pub(crate) admin: Option<PublicKey>,
  pub(crate) audiences: Vec<String>,
}

enum SpawnConfig {
  Http,
  Https,
  Redirect(String),
}

#[derive(Debug, Parser, PartialEq)]
pub(crate) struct Serve {
  #[arg(
    help = "Store ACME TLS certificates in <PATH>",
    long,
    value_name = "PATH"
  )]
  acme_cache: Option<Utf8PathBuf>,
  #[arg(
    help = "Provide ACME contact <CONTACT>, email addresses must include `mailto:` prefix",
    long,
    value_name = "CONTACT"
  )]
  acme_contact: Vec<String>,
  #[arg(
    default_value = "0.0.0.0",
    help = "Listen on <ADDRESS> for incoming requests",
    long
  )]
  address: String,
  #[arg(
    help = "Admin public key",
    long,
    requires = "restrict_uploads",
    value_name = "KEY"
  )]
  admin_key: Option<KeyIdentifier>,
  #[arg(
    help = "Request ACME TLS certificate and accept authorization tokens for <DOMAIN>, as well as \
            redirect HTTP to HTTPS at <DOMAIN> if enabled, this server must be reachable at \
            <DOMAIN>:443 to respond to Encrypt ACME challenges",
    long = "domain",
    value_name = "DOMAIN"
  )]
  domains: Vec<String>,
  #[arg(help = "Serve HTTP traffic", long)]
  http: bool,
  #[arg(
    help = "Listen on <PORT> for incoming HTTP requests [default: 80]",
    long,
    value_name = "PORT"
  )]
  http_port: Option<u16>,
  #[arg(help = "Serve HTTPS traffic", long)]
  https: bool,
  #[arg(
    help = "Listen on <PORT> for incoming HTTPS requests [default: 443]",
    long,
    value_name = "PORT"
  )]
  https_port: Option<u16>,
  #[arg(
    help = "Write listening port to <ADDRESS>",
    long,
    value_name = "ADDRESS"
  )]
  ready_address: Option<SocketAddr>,
  #[arg(help = "Redirect HTTP to HTTPS", long)]
  redirect_http_to_https: bool,
  #[arg(help = "Restrict uploads to admin", long)]
  restrict_uploads: bool,
}

impl Serve {
  fn acceptor(&self, acme_cache: Utf8PathBuf) -> Result<AxumAcceptor> {
    static RUSTLS_PROVIDER_INSTALLED: LazyLock<bool> = LazyLock::new(|| {
      rustls::crypto::ring::default_provider()
        .install_default()
        .is_ok()
    });

    ensure!(*RUSTLS_PROVIDER_INSTALLED, error::RustlsProvider);

    let config = AcmeConfig::new(self.domains()?)
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

  fn domains(&self) -> Result<Vec<String>> {
    if self.domains.is_empty() {
      Ok(vec![System::host_name().context(error::AcmeHostname)?])
    } else {
      Ok(self.domains.clone())
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

  fn redirect_destination(domains: &[String], https_port: u16) -> String {
    if https_port == 443 {
      format!("https://{}", domains[0])
    } else {
      format!("https://{}:{https_port}", domains[0])
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

  pub(crate) fn router(server: Arc<Server>, auth_config: Option<Arc<AuthConfig>>) -> Router {
    let router = Router::new()
      .route("/{hash}", get(Self::download))
      .route("/{hash}", put(Self::upload))
      .layer(Extension(server));

    if let Some(auth_config) = auth_config {
      router.layer(Extension(auth_config))
    } else {
      router
    }
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

    let server = Arc::new(Server::with_data_dir(&options.data_dir()?)?);

    let auth_config = if self.restrict_uploads {
      let admin = if let Some(identifier) = &self.admin_key {
        Some(Keychain::load(&options)?.identifier_public_key(identifier)?)
      } else {
        None
      };
      Some(Arc::new(AuthConfig {
        admin,
        audiences: self.domains()?,
      }))
    } else {
      None
    };

    let router = Self::router(server, auth_config);

    match (self.http_port(), self.https_port()) {
      (Some(http_port), None) => {
        self
          .spawn(SpawnConfig::Http, handle, &options, http_port, true, router)
          .await?;
      }
      (None, Some(https_port)) => {
        self
          .spawn(
            SpawnConfig::Https,
            handle,
            &options,
            https_port,
            true,
            router,
          )
          .await?;
      }
      (Some(http_port), Some(https_port)) => {
        let http_spawn_config = if self.redirect_http_to_https {
          SpawnConfig::Redirect(Self::redirect_destination(&self.domains()?, https_port))
        } else {
          SpawnConfig::Http
        };

        tokio::try_join!(
          self.spawn(
            http_spawn_config,
            handle.clone(),
            &options,
            http_port,
            true,
            router.clone(),
          ),
          self.spawn(
            SpawnConfig::Https,
            handle,
            &options,
            https_port,
            false,
            router,
          ),
        )?;
      }
      (None, None) => unreachable!(),
    }

    Ok(())
  }

  async fn spawn(
    &self,
    config: SpawnConfig,
    handle: Handle<SocketAddr>,
    options: &Options,
    port: u16,
    ready: bool,
    router: Router,
  ) -> Result {
    let listener = TcpListener::bind((self.address.as_str(), port))
      .await
      .context(error::BindListener {
        address: if self.address.contains(':') {
          format!("[{}]:{port}", self.address)
        } else {
          format!("{}:{port}", self.address)
        },
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
      SpawnConfig::Https => {
        let data_dir = options.data_dir()?;

        let acme_cache = self
          .acme_cache
          .clone()
          .unwrap_or_else(|| data_dir.join("acme-cache"));

        axum_server::from_tcp(listener)
          .context(error::Serve)?
          .handle(handle)
          .acceptor(self.acceptor(acme_cache)?)
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

  async fn upload(
    _: Authenticated,
    server: Extension<Arc<Server>>,
    hash: Path<Hash>,
    body: Body,
  ) -> ServerResult {
    server.write_file(*hash, body).await
  }
}

impl Default for Serve {
  fn default() -> Self {
    Self {
      acme_cache: None,
      acme_contact: Vec::new(),
      address: "0.0.0.0".into(),
      admin_key: None,
      domains: Vec::new(),
      http: false,
      http_port: None,
      https: false,
      https_port: None,
      ready_address: None,
      redirect_http_to_https: false,
      restrict_uploads: false,
    }
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
      Self::with_auth(None)
    }

    async fn put(&self, path: &str, content: &[u8]) -> Response {
      self.put_with_token(path, content, None).await
    }

    async fn put_with_token(&self, path: &str, content: &[u8], token: Option<&str>) -> Response {
      let mut request = Request::builder().method("PUT").uri(path);
      if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
      }
      self
        .router
        .clone()
        .oneshot(request.body(Body::from(content.to_vec())).unwrap())
        .await
        .unwrap()
    }

    fn with_auth(auth_config: Option<Arc<AuthConfig>>) -> Self {
      let (tempdir, data_dir) = tempdir();

      let server = Arc::new(Server::with_data_dir(&data_dir).unwrap());

      let router = Serve::router(server, auth_config);

      Self {
        data_dir,
        router,
        tempdir,
      }
    }

    fn write_file(&self, hash: Hash, content: &[u8]) {
      fs::write(self.data_dir.join("files").join(hash.to_string()), content).unwrap();
    }
  }

  #[test]
  fn admin_key_requires_restrict_upload() {
    let err = Serve::try_parse_from(["filepack", "--admin-key", test::PUBLIC_KEY]).unwrap_err();
    assert_eq!(err.kind(), clap::error::ErrorKind::MissingRequiredArgument);
  }

  #[tokio::test]
  async fn closed_server_forbids_uploads() {
    let server = TestServer::with_auth(Some(Arc::new(AuthConfig {
      admin: None,
      audiences: Vec::new(),
    })));

    let hash = Hash::bytes(b"bar");
    let response = server.put(&format!("/{hash}"), b"bar").await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
  }

  #[test]
  fn default_serve_matches_parsed() {
    assert_eq!(
      Serve::default(),
      Serve::try_parse_from(["filepack"]).unwrap(),
    );
  }

  #[test]
  fn domain_defaults_to_hostname() {
    assert_eq!(
      Serve::default().domains().unwrap(),
      vec![System::host_name().unwrap()]
    );
  }

  #[test]
  fn domain_flag_is_respected() {
    assert_eq!(
      Serve {
        domains: vec!["foo".into(), "bar".into()],
        ..Serve::default()
      }
      .domains()
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

    case(Serve::default(), Some(80), None);
    case(
      Serve {
        https: true,
        ..Serve::default()
      },
      None,
      Some(443),
    );
    case(
      Serve {
        https_port: Some(433),
        ..Serve::default()
      },
      None,
      Some(433),
    );
    case(
      Serve {
        http: true,
        https: true,
        ..Serve::default()
      },
      Some(80),
      Some(443),
    );
    case(
      Serve {
        http_port: Some(8080),
        https_port: Some(8443),
        ..Serve::default()
      },
      Some(8080),
      Some(8443),
    );
    case(
      Serve {
        redirect_http_to_https: true,
        ..Serve::default()
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

  #[tokio::test]
  async fn restricted_upload_accepts_admin_token() {
    let admin = PrivateKey::generate();
    let server = TestServer::with_auth(Some(Arc::new(AuthConfig {
      admin: Some(admin.public_key()),
      audiences: vec!["filepack.example".into()],
    })));

    let hash = Hash::bytes(b"bar");
    let token = Token::encode(&admin, "filepack.example").unwrap();

    let response = server
      .put_with_token(&format!("/{hash}"), b"bar", Some(&token))
      .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
      fs::read(server.data_dir.join("files").join(hash.to_string())).unwrap(),
      b"bar",
    );
  }

  #[tokio::test]
  async fn restricted_upload_rejects_missing_header() {
    let admin = PrivateKey::generate();
    let server = TestServer::with_auth(Some(Arc::new(AuthConfig {
      admin: Some(admin.public_key()),
      audiences: vec!["filepack.example".into()],
    })));

    let hash = Hash::bytes(b"bar");
    let response = server.put(&format!("/{hash}"), b"bar").await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
  }

  #[tokio::test]
  async fn restricted_upload_rejects_others() {
    let admin = PrivateKey::generate();
    let other = PrivateKey::generate();
    let server = TestServer::with_auth(Some(Arc::new(AuthConfig {
      admin: Some(admin.public_key()),
      audiences: vec!["filepack.example".into()],
    })));

    let hash = Hash::bytes(b"bar");
    let token = Token::encode(&other, "filepack.example").unwrap();

    let response = server
      .put_with_token(&format!("/{hash}"), b"bar", Some(&token))
      .await;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
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
