use {
  super::*,
  axum::{
    Router,
    body::Bytes,
    extract::{Extension, Path},
    routing::{get, put},
  },
  axum_server::Handle,
  clap::value_parser,
  tokio::{net::TcpListener, runtime},
};

// todo:
// - make sure I only have a single TLS crate and crypto provider in-tree
//
// test:
// - http1 and http2 are supported
// - download fails if file already exists

static HANDLE: LazyLock<Handle<SocketAddr>> = LazyLock::new(|| Handle::new());
static THREAD_COUNTER: AtomicU64 = AtomicU64::new(0);
static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

#[derive(Parser)]
pub(crate) struct Serve {
  #[arg(help = "Listen on <ADDRESS> for incoming requests.", long)]
  address: String,
  #[arg(
    help = "Write local listening port to file descriptor <READY_FD>, which must be open and be passed by the caller.",
    long,
    value_parser = value_parser!(RawFd).range(3..),
  )]
  ready_fd: Option<RawFd>,
}

impl Serve {
  pub(crate) fn run(self, options: Options) -> Result {
    let runtime = runtime::Builder::new_multi_thread()
      .name("server")
      .thread_name_fn(|| {
        format!(
          "server-thread-{}",
          THREAD_COUNTER.fetch_add(1, atomic::Ordering::Relaxed)
        )
      })
      .enable_io()
      .build()
      .context(error::ServerRuntime)?;

    runtime.block_on(self.serve(options))?;

    Ok(())
  }

  async fn serve(self, options: Options) -> Result {
    ctrlc::set_handler(move || {
      if SHUTTING_DOWN.fetch_or(true, atomic::Ordering::Relaxed) {
        process::exit(1);
      }

      HANDLE.graceful_shutdown(Some(Duration::from_millis(100)));
    })
    .unwrap();

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

    if let Some(fd) = self.ready_fd {
      let local_address = listener.local_addr().context(error::LocalAddress)?;

      let port = local_address.port().to_string();

      let mut written = 0;
      while written < port.len() {
        let result = unsafe { libc::write(fd, port.as_ptr().cast(), port.len()) };

        if result < 0 {
          return Err(error::ReadyFd.into_error(io::Error::last_os_error()));
        }

        written += usize::try_from(result).unwrap();
      }

      let result = unsafe { libc::close(fd) };

      if result < 0 {
        return Err(error::ReadyFd.into_error(io::Error::last_os_error()));
      }
    }

    axum_server::from_tcp(listener)
      .unwrap()
      .handle(HANDLE.clone())
      .serve(router.into_make_service())
      .await
      .context(error::Serve)?;

    Ok(())
  }

  async fn download(server: Extension<Arc<Server>>, hash: Path<Hash>) -> ServerResult<Vec<u8>> {
    server.read_file(*hash)
  }

  async fn upload(server: Extension<Arc<Server>>, hash: Path<Hash>, file: Bytes) -> ServerResult {
    server.write_file(*hash, &file)
  }
}
