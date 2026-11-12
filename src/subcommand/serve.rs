use {
  self::{
    host_port::HostPort, packages_query::PackagesQuery, redirect_config::RedirectConfig,
    scheme::Scheme, server_config::ServerConfig,
  },
  super::*,
  axum::{
    Router, body,
    extract::{Extension, Path, Query, Request},
    http::{HeaderValue, Method, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::get,
  },
  axum_extra::headers::HeaderMapExt,
  futures_util::stream,
  templates::{
    DirectoryHtml, FilesHtml, HistoryHtml, HomeHtml, ItemHtml, MediaHtml, PackageHtml, PackagesHtml,
  },
  tokio::task::block_in_place,
  tower_http::set_header::SetResponseHeaderLayer,
};

#[cfg(feature = "server")]
use {
  self::spawn_config::SpawnConfig,
  axum_server::Handle,
  hyper_util::rt::TokioTimer,
  rustls_acme::{
    AcmeConfig, EventOk, acme::LETS_ENCRYPT_PRODUCTION_DIRECTORY, axum::AxumAcceptor,
    caches::DirCache,
  },
  tokio::{net::TcpListener, runtime},
};

pub(crate) use self::write_config::WriteConfig;

mod host_port;
mod packages_query;
mod redirect_config;
pub(crate) mod route;
mod scheme;
mod server_config;
#[cfg(feature = "server")]
mod spawn_config;
#[cfg(test)]
mod tests;
mod write_config;

const CROSS_ORIGIN_RESOURCE_POLICY: HeaderName =
  HeaderName::from_static("cross-origin-resource-policy");
#[cfg(not(feature = "server"))]
const LETS_ENCRYPT_PRODUCTION_DIRECTORY: &str = "https://acme-v02.api.letsencrypt.org/directory";
const SEC_FETCH_SITE: HeaderName = HeaderName::from_static("sec-fetch-site");

#[cfg(feature = "server")]
static THREAD_COUNTER: AtomicU64 = AtomicU64::new(0);

type HostPortExtension = Extension<HostPort>;
type RedirectConfigExtension = Extension<Arc<RedirectConfig>>;
type SchemeExtension = Extension<Scheme>;
type ServerConfigExtension = Extension<Arc<ServerConfig>>;
type ServerExtension = Extension<Arc<Server>>;

#[derive(Debug, Parser, PartialEq)]
pub(crate) struct Serve {
  #[arg(
    help = "Store ACME TLS certificates in <PATH>",
    long,
    value_hint = ValueHint::DirPath,
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
    default_value = LETS_ENCRYPT_PRODUCTION_DIRECTORY,
    env = "FILEPACK_ACME_DIRECTORY",
    help = "Request ACME TLS certificates from <DIRECTORY>",
    long,
    value_name = "DIRECTORY"
  )]
  acme_directory: Url,
  #[arg(
    default_value = "127.0.0.1",
    help = "Listen on <ADDRESS> for incoming requests",
    long
  )]
  address: IpAddr,
  #[arg(
    help = "Admin public key or key name",
    long,
    requires = "domain",
    value_name = "KEY"
  )]
  admin_key: Option<KeyIdentifier>,
  #[arg(
    help = "Use <DOMAIN> as canonical domain: request ACME TLS certificates for it, accept \
            authentication tokens scoped to it, and redirect to it",
    long,
    value_name = "DOMAIN"
  )]
  domain: Option<Domain>,
  #[arg(help = "Serve HTTP traffic", long)]
  http: bool,
  #[arg(
    help = "Listen on <PORT> for incoming HTTP requests [default: 80]",
    long,
    value_name = "PORT"
  )]
  http_port: Option<u16>,
  #[arg(help = "Serve HTTPS traffic", long, requires = "domain")]
  https: bool,
  #[arg(
    help = "Listen on <PORT> for incoming HTTPS requests [default: 443]",
    long,
    requires = "domain",
    value_name = "PORT"
  )]
  https_port: Option<u16>,
  #[arg(
    help = "Mount web package <PACKAGE>. <PACKAGE> may be a fingerprint, revision, or number. \
            Mounted packages are served under `/mount` by fingerprint, revision, and number, as \
            well as directly under `/<NAME>/` if `<NAME>` is supplied. Mounted packages are \
            unsandboxed and served with the server origin. Make sure you understand the \
            consequences of this and only mount packages you trust",
    long = "mount",
    value_name = "PACKAGE[:NAME]"
  )]
  mounts: Vec<Mount>,
  #[arg(
    help = "Write listening port to <ADDRESS>",
    long,
    value_name = "ADDRESS"
  )]
  ready_address: Option<SocketAddr>,
  #[arg(help = "Redirect HTTP to HTTPS", long, requires = "domain")]
  redirect_http_to_https: bool,
  #[arg(
    help = "Redirect requests for <DOMAIN> to the canonical domain, and request ACME TLS \
            certificates for it",
    long = "redirect",
    requires = "domain",
    value_name = "DOMAIN"
  )]
  redirects: Vec<Domain>,
  #[arg(
    conflicts_with = "admin_key",
    help = "Allow unauthenticated writes",
    long
  )]
  unrestricted_writes: bool,
}

pub(crate) type Input = ();

pub(crate) type Output = ();

impl Serve {
  #[cfg(feature = "server")]
  fn acceptor(&self, acme_cache: Utf8PathBuf) -> Result<AxumAcceptor> {
    install_default_crypto_provider()?;

    let config = AcmeConfig::new(self.domains().iter().map(Domain::as_str))
      .contact(&self.acme_contact)
      .cache_option(Some(DirCache::new(acme_cache)))
      .directory(&self.acme_directory);

    let mut state = config.state();

    let mut server_config = rustls::ServerConfig::builder()
      .with_no_client_auth()
      .with_cert_resolver(state.resolver());

    server_config.alpn_protocols = vec!["h2".into(), "http/1.1".into()];

    let acceptor = state.axum_acceptor(Arc::new(server_config));

    tokio::spawn(async move {
      while let Some(result) = state.next().await {
        match result {
          Ok(event) => {
            let event = match event {
              EventOk::AccountCacheStore => "cached new account credentials",
              EventOk::CertCacheStore => "cached new certificate",
              EventOk::DeployedCachedCert => "deployed cached certificate",
              EventOk::DeployedNewCert => "deployed new certificate",
            };
            tracing::info!("ACME event: {event}");
          }
          Err(err) => tracing::error!("ACME error: {err}"),
        }
      }
    });

    Ok(acceptor)
  }

  async fn api_fallback() -> ServerError {
    ServerError::ApiEndpointNotFound
  }

  async fn api_version_layer(request: Request, next: Next) -> Response {
    let version = if let Some(value) = request.headers().get(API_VERSION_HEADER) {
      let Some(version) = value
        .to_str()
        .ok()
        .and_then(|value| parse_number::<u64>(value).ok())
      else {
        return ServerError::ApiVersionMalformed {
          value: value.as_bytes().escape_ascii().to_string(),
        }
        .into_response();
      };
      version
    } else {
      0
    };

    if version != API_VERSION {
      return ServerError::ApiVersionUnsupported { version }.into_response();
    }

    next.run(request).await
  }

  fn canonical(&self) -> &Domain {
    self.domain.as_ref().unwrap()
  }

  async fn cross_origin_layer(
    host_port: HostPortExtension,
    scheme: SchemeExtension,
    request: Request,
    next: Next,
  ) -> Response {
    if matches!(
      *request.method(),
      Method::GET | Method::HEAD | Method::OPTIONS
    ) {
      return next.run(request).await;
    }

    let headers = request.headers();

    if let Some(site) = headers.get(SEC_FETCH_SITE) {
      return if site == "same-origin" || site == "none" {
        next.run(request).await
      } else {
        ServerError::CrossOriginRequest.into_response()
      };
    }

    let Ok(origin) = headers.typed_try_get::<headers::Origin>() else {
      return ServerError::CrossOriginRequest.into_response();
    };

    let Some(origin) = origin else {
      return next.run(request).await;
    };

    if origin.scheme() != scheme.name() {
      return ServerError::CrossOriginRequest.into_response();
    }

    let Ok(host) = origin.hostname().parse::<Host>() else {
      return ServerError::CrossOriginRequest.into_response();
    };

    let origin = HostPort {
      host,
      port: origin.port(),
    };

    if origin != *host_port {
      return ServerError::CrossOriginRequest.into_response();
    }

    next.run(request).await
  }

  fn domains(&self) -> Vec<Domain> {
    self.domain.iter().chain(&self.redirects).cloned().collect()
  }

  async fn fallback(server_config: ServerConfigExtension, uri: Uri) -> Result<Redirect, PageError> {
    if let Some(segment) = uri.path().strip_prefix('/')
      && !segment.contains('/')
    {
      if segment.starts_with(|c: char| c.is_ascii_digit()) {
        let number = parse_number::<u64>(segment).context(server_error::NumberParse)?;
        return Ok(Redirect::permanent(&format!("/package/{number}")));
      } else if segment.starts_with(&Tag::Package.prefix()) {
        let fingerprint = segment
          .parse::<Fingerprint>()
          .context(server_error::FingerprintParse)?;
        return Ok(Redirect::permanent(&format!("/package/{fingerprint}")));
      } else if segment.starts_with(&Tag::Revision.prefix()) {
        let revision = segment
          .parse::<Revision>()
          .context(server_error::RevisionParse)?;
        return Ok(Redirect::permanent(&format!("/package/{revision}")));
      } else if server_config.mount(segment).is_some() {
        return Ok(Redirect::temporary(&format!("/{segment}/")));
      }
    }

    Err(ServerError::PageNotFound.into())
  }

  async fn host_layer(
    server_config: ServerConfigExtension,
    header: Option<TypedHeader<headers::Host>>,
    mut request: Request,
    next: Next,
  ) -> Response {
    let Some((hostname, port)) = request
      .uri()
      .host()
      .map(|host| (host, request.uri().port_u16()))
      .or(
        header
          .as_ref()
          .map(|header| (header.hostname(), header.port())),
      )
    else {
      return ServerError::HostMissing.into_response();
    };

    let Ok(host) = hostname.parse::<Host>() else {
      return ServerError::HostNotAllowed {
        host: hostname.into(),
      }
      .into_response();
    };

    if let Host::Domain(domain) = &host
      && domain.as_str() != "localhost"
      && !server_config.domains.contains(domain)
    {
      return ServerError::HostNotAllowed {
        host: hostname.into(),
      }
      .into_response();
    }

    request.extensions_mut().insert(HostPort { host, port });

    next.run(request).await
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

  async fn if_none_match_layer(request: Request, next: Next) -> Response {
    let if_none_match = matches!(*request.method(), Method::GET | Method::HEAD)
      .then(|| request.headers().typed_get::<headers::IfNoneMatch>())
      .flatten();

    let mut response = next.run(request).await;

    if matches!(
      response.status(),
      StatusCode::OK | StatusCode::PARTIAL_CONTENT
    ) && let Some(if_none_match) = if_none_match
      && let Some(etag) = response.headers().typed_get::<headers::ETag>()
      && !if_none_match.precondition_passes(&etag)
    {
      *response.status_mut() = StatusCode::NOT_MODIFIED;
      *response.body_mut() = Body::from_stream(stream::empty::<Result<body::Bytes, Infallible>>());

      for name in [
        header::CONTENT_LENGTH,
        header::CONTENT_RANGE,
        header::CONTENT_TYPE,
      ] {
        response.headers_mut().remove(name);
      }
    }

    response
  }

  async fn named_mount(
    server: ServerExtension,
    server_config: ServerConfigExtension,
    Path(name): Path<String>,
    range: Option<TypedHeader<headers::Range>>,
  ) -> Result<Resource, PageError> {
    Self::named_mount_with_path(
      server,
      server_config,
      Path((name, "index.html".parse().unwrap())),
      range,
    )
    .await
  }

  async fn named_mount_with_path(
    server: ServerExtension,
    server_config: ServerConfigExtension,
    Path((name, path)): Path<(String, RelativePath)>,
    range: Option<TypedHeader<headers::Range>>,
  ) -> Result<Resource, PageError> {
    let Some(mount) = server_config.mount(&name) else {
      return Err(ServerError::PageNotFound.into());
    };

    Ok(
      block_in_place(|| server.mount_file(false, &server_config.mounts, mount.package, &path))?
        .range(range),
    )
  }

  fn redirect_config(&self) -> Result<Option<Arc<RedirectConfig>>> {
    if self.redirects.is_empty() {
      return Ok(None);
    }

    let canonical = self.canonical();

    for redirect in &self.redirects {
      ensure!(
        redirect != canonical,
        error::RedirectDomainCanonical {
          domain: redirect.clone()
        },
      );
    }

    Ok(Some(Arc::new(RedirectConfig {
      destination: self.redirect_url(),
      domains: self.redirects.iter().cloned().collect(),
    })))
  }

  async fn redirect_http_to_https(redirect_config: RedirectConfigExtension, uri: Uri) -> Redirect {
    Redirect::permanent(redirect_config.with_path_and_query(&uri).as_str())
  }

  async fn redirect_layer(
    redirect_config: RedirectConfigExtension,
    host_port: HostPortExtension,
    request: Request,
    next: Next,
  ) -> Response {
    if let Host::Domain(domain) = &host_port.host
      && redirect_config.domains.contains(domain)
    {
      Redirect::permanent(redirect_config.with_path_and_query(request.uri()).as_str())
        .into_response()
    } else {
      next.run(request).await
    }
  }

  fn redirect_url(&self) -> Url {
    let domain = self.canonical();

    if let Some(port) = self.https_port() {
      if port == 443 {
        format!("https://{domain}/")
      } else {
        format!("https://{domain}:{port}/")
      }
    } else {
      let port = self.http_port().unwrap();
      if port == 80 {
        format!("http://{domain}/")
      } else {
        format!("http://{domain}:{port}/")
      }
    }
    .parse()
    .unwrap()
  }

  pub(crate) fn router(
    server: Arc<Server>,
    redirect_config: Option<Arc<RedirectConfig>>,
    server_config: Arc<ServerConfig>,
    write_config: Arc<WriteConfig>,
  ) -> Router {
    let mut api = Router::new();
    let mut web = Router::new();

    for route in Route::all(&mut BTreeMap::new()) {
      if let Some(path) = route.api_path {
        api = api.route(path, route.router);
      } else {
        web = web.route(route.path, route.router);
      }
    }

    let router = web
      .nest(
        "/api",
        api
          .fallback(Self::api_fallback)
          .layer(middleware::from_fn(Self::api_version_layer))
          .layer(SetResponseHeaderLayer::overriding(
            API_VERSION_HEADER,
            HeaderValue::from(API_VERSION),
          )),
      )
      .fallback_service(
        Router::new()
          .route("/{name}/", get(Self::named_mount))
          .route("/{name}/{*path}", get(Self::named_mount_with_path))
          .fallback(Self::fallback),
      )
      .layer(Extension(server))
      .layer(Extension(write_config))
      .layer(middleware::from_fn(Self::if_none_match_layer));

    let router = if let Some(redirect_config) = redirect_config {
      router
        .layer(middleware::from_fn(Self::redirect_layer))
        .layer(Extension(redirect_config))
    } else {
      router
    };

    router
      .layer(middleware::from_fn(Self::cross_origin_layer))
      .layer(middleware::from_fn(Self::host_layer))
      .layer(Extension(server_config))
      .layer(SetResponseHeaderLayer::appending(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("frame-ancestors 'self'"),
      ))
      .layer(SetResponseHeaderLayer::overriding(
        CROSS_ORIGIN_RESOURCE_POLICY,
        HeaderValue::from_static("same-origin"),
      ))
      .layer(SetResponseHeaderLayer::overriding(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
      ))
  }

  #[cfg(feature = "server")]
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

  #[cfg(feature = "server")]
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

    let server = Arc::new(Server::with_data_dir(
      &options.data_dir()?,
      options.unstable,
    )?);

    let write_config = if let Some(identifier) = &self.admin_key {
      WriteConfig::Admin {
        domain: self.canonical().clone(),
        key: Keychain::load(&options)?.identifier_public_key(identifier)?,
      }
    } else if self.unrestricted_writes {
      WriteConfig::Unrestricted
    } else {
      WriteConfig::Forbidden
    };

    let mut names = HashSet::new();

    for name in self.mounts.iter().filter_map(|mount| mount.name.as_deref()) {
      ensure!(names.insert(name), error::MountNameDuplicate { name });
    }

    let redirect_config = self.redirect_config()?;

    let router = Self::router(
      server,
      redirect_config,
      self.server_config(),
      Arc::new(write_config),
    );

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
          SpawnConfig::Redirect(self.redirect_url())
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

  #[cfg(feature = "server")]
  fn server(
    handle: Handle<SocketAddr>,
    listener: std::net::TcpListener,
  ) -> Result<axum_server::Server<SocketAddr>> {
    let mut server = axum_server::from_tcp(listener).context(error::Serve)?;
    let builder = server.http_builder();
    builder.http1().timer(TokioTimer::new());
    builder
      .http2()
      .adaptive_window(true)
      .keep_alive_interval(Duration::from_mins(1))
      .keep_alive_timeout(Duration::from_secs(20))
      .max_frame_size(u32::try_from(64 * KIB).unwrap())
      .timer(TokioTimer::new());
    Ok(server.handle(handle))
  }

  fn server_config(&self) -> Arc<ServerConfig> {
    Arc::new(ServerConfig {
      domains: self.domains().into_iter().collect(),
      mounts: self.mounts.clone(),
      url: self.domain.as_ref().map(|_| self.redirect_url()),
    })
  }

  #[cfg(feature = "server")]
  async fn spawn(
    &self,
    config: SpawnConfig,
    handle: Handle<SocketAddr>,
    options: &Options,
    port: u16,
    ready: bool,
    router: Router,
  ) -> Result {
    let address = SocketAddr::new(self.address, port);

    let listener = TcpListener::bind(address)
      .await
      .context(error::BindListener { address })?
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
        Self::server(handle, listener)?
          .serve(router.layer(Extension(Scheme::Http)).into_make_service())
          .await
          .context(error::Serve)?;
      }
      SpawnConfig::Https => {
        let data_dir = options.data_dir()?;

        let acme_cache = self
          .acme_cache
          .clone()
          .unwrap_or_else(|| data_dir.join("acme-cache"));

        let router =
          router
            .layer(Extension(Scheme::Https))
            .layer(SetResponseHeaderLayer::overriding(
              header::STRICT_TRANSPORT_SECURITY,
              HeaderValue::from_static("max-age=31536000"),
            ));

        Self::server(handle, listener)?
          .acceptor(self.acceptor(acme_cache)?)
          .serve(router.into_make_service())
          .await
          .context(error::Serve)?;
      }
      SpawnConfig::Redirect(destination) => {
        let router = Router::new()
          .fallback(Self::redirect_http_to_https)
          .layer(Extension(Arc::new(RedirectConfig {
            destination,
            domains: HashSet::new(),
          })))
          .layer(SetResponseHeaderLayer::overriding(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
          ));

        Self::server(handle, listener)?
          .serve(router.into_make_service())
          .await
          .context(error::Serve)?;
      }
    }

    Ok(())
  }
}

impl Default for Serve {
  fn default() -> Self {
    Self {
      acme_cache: None,
      acme_contact: Vec::new(),
      acme_directory: LETS_ENCRYPT_PRODUCTION_DIRECTORY.parse().unwrap(),
      address: Ipv4Addr::LOCALHOST.into(),
      admin_key: None,
      domain: None,
      http: false,
      http_port: None,
      https: false,
      https_port: None,
      mounts: Vec::new(),
      ready_address: None,
      redirect_http_to_https: false,
      redirects: Vec::new(),
      unrestricted_writes: false,
    }
  }
}
