use {
  super::*,
  axum::{
    Router,
    body::{Body, Bytes},
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
  async fn download(server: Extension<Arc<Server>>, hash: Path<Hash>) -> ServerResult<Response> {
    let (file, len) = server.open_file(*hash).await?;

    Ok(
      Response::builder()
        .header(header::CONTENT_LENGTH, len)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .body(Body::from_stream(ReaderStream::new(file)))
        .unwrap(),
    )
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

    let router = Router::new()
      .route("/{hash}", get(Self::download))
      .route("/{hash}", put(Self::upload))
      .layer(Extension(server));

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

  async fn upload(server: Extension<Arc<Server>>, hash: Path<Hash>, file: Bytes) -> ServerResult {
    server.write_file(*hash, &file)
  }
}
