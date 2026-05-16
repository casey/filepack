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

static THREAD_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Parser)]
pub(crate) struct Serve {
  #[arg(help = "Listen on <ADDRESS> for incoming requests", long)]
  address: String,
  #[arg(
    help = "Write local listening port to file descriptor <READY_FD>, which must be open and be passed by the caller",
    long,
    value_parser = value_parser!(RawFd).range(3..),
  )]
  ready_fd: Option<RawFd>,
}

impl Serve {
  async fn download(server: Extension<Arc<Server>>, hash: Path<Hash>) -> ServerResult<Vec<u8>> {
    server.read_file(*hash)
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

    if let Some(fd) = self.ready_fd {
      let local_address = listener.local_addr().context(error::LocalAddress)?;

      let port = local_address.port().to_string().into_bytes();

      let mut written = 0;
      while written < port.len() {
        let buffer = &port[written..];
        let result = unsafe { libc::write(fd, buffer.as_ptr().cast(), buffer.len()) };

        if result < 0 {
          let err = io::Error::last_os_error();

          if err.kind() == io::ErrorKind::Interrupted {
            continue;
          }

          return Err(error::ReadyFd.into_error(err));
        }

        written += usize::try_from(result).unwrap();
      }

      let result = unsafe { libc::close(fd) };

      if result < 0 {
        return Err(error::ReadyFd.into_error(io::Error::last_os_error()));
      }
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
