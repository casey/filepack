use {
  super::*,
  axum::{
    Router,
    extract::{Extension, Path},
    http::{HeaderValue, Uri, header},
    response::Redirect,
    routing::{get, post, put},
  },
  axum_server::Handle,
  rustls_acme::{
    AcmeConfig, acme::LETS_ENCRYPT_PRODUCTION_DIRECTORY, axum::AxumAcceptor, caches::DirCache,
  },
  std::net::TcpStream,
  sysinfo::System,
  templates::{DirectoryHtml, FilesHtml, PackageHtml},
  tokio::{net::TcpListener, runtime, task::block_in_place},
  tokio_util::io::ReaderStream,
  tower_http::set_header::SetResponseHeaderLayer,
};

static THREAD_COUNTER: AtomicU64 = AtomicU64::new(0);

type ServerExtension = Extension<Arc<Server>>;

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
            <DOMAIN>:443 to respond to Let's Encrypt ACME challenges",
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

  async fn directory(
    server: ServerExtension,
    Path(hash): Path<Hash>,
  ) -> ServerResult<DirectoryHtml> {
    Ok(DirectoryHtml {
      directory: block_in_place(|| server.directory(hash))?,
      hash,
    })
  }

  fn domains(&self) -> Result<Vec<String>> {
    if self.domains.is_empty() {
      Ok(vec![System::host_name().context(error::AcmeHostname)?])
    } else {
      Ok(self.domains.clone())
    }
  }

  async fn download(server: ServerExtension, Path(hash): Path<Hash>) -> ServerResult<Response> {
    let (file, len) = block_in_place(|| server.open_file(hash))?;

    Ok(
      Response::builder()
        .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
        .header(header::CONTENT_DISPOSITION, "attachment")
        .header(header::CONTENT_LENGTH, len)
        .header(header::CONTENT_SECURITY_POLICY, "sandbox")
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::ETAG, format!("\"{hash}\""))
        .body(Body::from_stream(ReaderStream::new(
          tokio::fs::File::from_std(file),
        )))
        .unwrap(),
    )
  }

  async fn fallback() -> ServerResult<StaticAsset> {
    Ok(StaticAsset::get("404.html")?.status(StatusCode::NOT_FOUND))
  }

  async fn favicon() -> ServerResult<StaticAsset> {
    StaticAsset::get("favicon.png")
  }

  async fn files(server: ServerExtension) -> ServerResult<FilesHtml> {
    Ok(FilesHtml {
      files: block_in_place(|| server.files())?,
    })
  }

  async fn home() -> ServerResult<StaticAsset> {
    StaticAsset::get("index.html")
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

  async fn install_script() -> ServerResult<StaticAsset> {
    StaticAsset::get("install.sh")
  }

  async fn package(server: ServerExtension, Path(hash): Path<Hash>) -> ServerResult<PackageHtml> {
    Ok(PackageHtml {
      hash,
      metadata: block_in_place(|| server.package(hash))?,
    })
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

  fn redirect_router(destination: String) -> Router {
    Router::new()
      .fallback(Self::redirect_http_to_https)
      .layer(Extension(destination))
      .layer(SetResponseHeaderLayer::overriding(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
      ))
  }

  pub(crate) fn router(server: Arc<Server>, auth_config: Option<Arc<AuthConfig>>) -> Router {
    let router = Router::new()
      .route("/", get(Self::home))
      .route("/directory/{hash}", get(Self::directory))
      .route("/directory/{hash}", post(Self::verify_directory))
      .route("/favicon.ico", get(Self::favicon))
      .route("/file/{hash}", get(Self::download))
      .route("/file/{hash}", put(Self::upload))
      .route("/files", get(Self::files))
      .route("/install.sh", get(Self::install_script))
      .route("/package/{hash}", get(Self::package))
      .route("/package/{hash}", post(Self::verify_package))
      .route("/static/{*path}", get(Self::static_asset))
      .fallback(Self::fallback)
      .layer(Extension(server))
      .layer(SetResponseHeaderLayer::overriding(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
      ));

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
          .serve(Self::redirect_router(destination).into_make_service())
          .await
          .context(error::Serve)?;
      }
    }

    Ok(())
  }

  async fn static_asset(path: Path<String>) -> ServerResult<StaticAsset> {
    StaticAsset::get(&path)
  }

  async fn upload(
    _: Authenticated,
    server: ServerExtension,
    hash: Path<Hash>,
    body: Body,
  ) -> ServerResult {
    server.write_file(*hash, body).await
  }

  async fn verify_directory(
    _: Authenticated,
    server: ServerExtension,
    hash: Path<Hash>,
  ) -> ServerResult {
    block_in_place(|| server.verify_directory(*hash))
  }

  async fn verify_package(
    _: Authenticated,
    server: ServerExtension,
    hash: Path<Hash>,
  ) -> ServerResult {
    block_in_place(|| server.verify_package(*hash))
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
mod tests;
