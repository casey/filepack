use {
  super::*,
  axum::{
    Router,
    extract::{Extension, Path, Request},
    http::{HeaderValue, Uri},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
  },
  axum_server::Handle,
  rustls_acme::{
    AcmeConfig, EventOk, acme::LETS_ENCRYPT_PRODUCTION_DIRECTORY, axum::AxumAcceptor,
    caches::DirCache,
  },
  std::net::TcpStream,
  templates::{
    AudioHtml, DirectoryHtml, FilesHtml, ImageHtml, PackageHtml, PackagesHtml, VideoHtml,
  },
  tokio::{net::TcpListener, runtime, task::block_in_place},
  tower_http::set_header::SetResponseHeaderLayer,
};

mod route;
#[cfg(test)]
mod tests;

static THREAD_COUNTER: AtomicU64 = AtomicU64::new(0);

type RedirectConfigExtension = Extension<Arc<RedirectConfig>>;
type ServerExtension = Extension<Arc<Server>>;

pub(crate) struct AuthConfig {
  pub(crate) admin: Option<PublicKey>,
  pub(crate) audience: Option<String>,
}

pub(crate) struct RedirectConfig {
  destination: Url,
  domains: HashSet<String>,
}

impl RedirectConfig {
  fn with_path_and_query(&self, uri: &Uri) -> Url {
    let mut destination = self.destination.clone();
    destination.set_path(uri.path());
    destination.set_query(uri.query());
    destination
  }
}

enum SpawnConfig {
  Http,
  Https,
  Redirect(Url),
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
    default_value = LETS_ENCRYPT_PRODUCTION_DIRECTORY,
    env = "FILEPACK_ACME_DIRECTORY",
    help = "Request ACME TLS certificates from <DIRECTORY>",
    long,
    value_name = "DIRECTORY"
  )]
  acme_directory: Url,
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
    help = "Use <DOMAIN> as canonical domain: request ACME TLS certificates for it, accept \
            authentication tokens scoped to it, and redirect to it",
    long,
    value_name = "DOMAIN"
  )]
  domain: Option<String>,
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
  redirects: Vec<String>,
  #[arg(help = "Restrict uploads to admin", long)]
  restrict_uploads: bool,
}

impl Serve {
  fn acceptor(&self, acme_cache: Utf8PathBuf) -> Result<AxumAcceptor> {
    install_default_crypto_provider()?;

    let config = AcmeConfig::new(self.domains())
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

  fn canonical(&self) -> &str {
    self.domain.as_deref().unwrap()
  }

  fn domains(&self) -> Vec<String> {
    let mut domains = vec![self.canonical().to_string()];
    domains.extend(self.redirects.iter().cloned());
    domains
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

  fn redirect_config(&self) -> Result<Option<Arc<RedirectConfig>>> {
    if self.redirects.is_empty() {
      return Ok(None);
    }

    let canonical = self.canonical();

    for redirect in &self.redirects {
      ensure!(
        redirect.as_str() != canonical,
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
    host: Option<TypedHeader<headers::Host>>,
    request: Request,
    next: Next,
  ) -> Response {
    if let Some(host) = request
      .uri()
      .host()
      .or(host.as_ref().map(|TypedHeader(host)| host.hostname()))
      && redirect_config
        .domains
        .iter()
        .any(|domain| domain.eq_ignore_ascii_case(host))
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
    auth_config: Option<Arc<AuthConfig>>,
    redirect_config: Option<Arc<RedirectConfig>>,
  ) -> Router {
    let router = Router::new()
      .route("/", get(route::home))
      .route("/artwork/{fingerprint}", get(route::artwork))
      .route(
        "/directory/{hash}",
        get(route::directory).post(route::verify_directory),
      )
      .route("/favicon.ico", get(route::favicon))
      .route("/file/{hash}", get(route::file).put(route::upload_file))
      .route("/file/{hash}/{name}", get(route::file_with_name))
      .route("/files", get(route::files))
      .route("/install.sh", get(route::install_script))
      .route(
        "/media/audio/{fingerprint}/track/{track}",
        get(route::media_audio_item),
      )
      .route(
        "/media/image/{fingerprint}/image/{image}",
        get(route::media_image_item),
      )
      .route(
        "/media/video/{fingerprint}/video/{video}",
        get(route::media_video_item),
      )
      .route("/missing", post(route::missing))
      .route(
        "/package/{fingerprint}",
        get(route::package).post(route::verify_package),
      )
      .route("/package/{fingerprint}/{item}", get(route::package_item))
      .route("/packages", get(route::packages))
      .route("/static/{*path}", get(route::static_asset))
      .fallback(route::fallback)
      .layer(Extension(server))
      .layer(SetResponseHeaderLayer::overriding(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
      ));

    let router = if let Some(auth_config) = auth_config {
      router.layer(Extension(auth_config))
    } else {
      router
    };

    if let Some(redirect_config) = redirect_config {
      router
        .layer(middleware::from_fn(Self::redirect_layer))
        .layer(Extension(redirect_config))
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
        audience: self.domain.clone(),
      }))
    } else {
      None
    };

    let redirect_config = self.redirect_config()?;

    let router = Self::router(server, auth_config, redirect_config);

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

        let router = router.layer(SetResponseHeaderLayer::overriding(
          header::STRICT_TRANSPORT_SECURITY,
          HeaderValue::from_static("max-age=31536000"),
        ));

        axum_server::from_tcp(listener)
          .context(error::Serve)?
          .handle(handle)
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

        axum_server::from_tcp(listener)
          .context(error::Serve)?
          .handle(handle)
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
      address: "0.0.0.0".into(),
      admin_key: None,
      domain: None,
      http: false,
      http_port: None,
      https: false,
      https_port: None,
      ready_address: None,
      redirect_http_to_https: false,
      redirects: Vec::new(),
      restrict_uploads: false,
    }
  }
}
