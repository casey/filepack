use {
  super::*,
  axum::{
    Router,
    body::Bytes,
    extract::{Extension, Path},
    routing::{get, put},
  },
};

// todo:
// - should use multi-threading in production and current thread in tests?
//   should i wait and benchmark this?
// - avoid copying whole file into memory
// - use in-process server tests
// - write an error string to ready_fd if we fail to start
// - test:
//   - http2 is supported

static LISTENERS: Mutex<Vec<axum_server::Handle<SocketAddr>>> = Mutex::new(Vec::new());

static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

struct Server {
  files: Utf8PathBuf,
}

impl Server {
  fn read_file(&self, hash: Hash) -> NodeResult<Vec<u8>> {
    let path = self.files.join(hash.to_string());
    fs::read(&path).context(node_error::FilesystemIo { path })
  }

  fn write_file(&self, hash: Hash, contents: &[u8]) -> NodeResult {
    let path = self.files.join(hash.to_string());

    let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
      Ok(file) => file,
      Err(err) => {
        if err.kind() == io::ErrorKind::AlreadyExists {
          return Ok(());
        }

        return Err(node_error::FilesystemIo { path }.into_error(err));
      }
    };

    file
      .write_all(contents)
      .context(node_error::FilesystemIo { path })
  }
}

#[derive(Parser)]
pub(crate) struct Serve {
  address: String,
  #[arg(long)]
  ready_fd: Option<std::os::fd::RawFd>,
}

impl Serve {
  pub(crate) fn run(self, options: Options) -> Result {
    let runtime = tokio::runtime::Builder::new_current_thread()
      .enable_io()
      .build()
      .unwrap();

    runtime.block_on(self.serve(options)).unwrap();

    Ok(())
  }

  async fn serve(self, options: Options) -> Result {
    let handle = axum_server::Handle::new();

    LISTENERS.lock().unwrap().push(handle.clone());

    ctrlc::set_handler(move || {
      if SHUTTING_DOWN.fetch_or(true, atomic::Ordering::Relaxed) {
        process::exit(1);
      }

      LISTENERS
        .lock()
        .unwrap()
        .iter()
        .for_each(|handle| handle.graceful_shutdown(Some(Duration::from_millis(100))));
    })
    .expect("failed to set <CTRL-C> handler");

    let server = Arc::new(Server {
      files: options.data_dir()?.join("files"),
    });

    let router = Router::new()
      .route("/{hash}", get(Self::get_file))
      .route("/{hash}", put(Self::put_file))
      .layer(Extension(server));

    let listener = tokio::net::TcpListener::bind(&self.address)
      .await
      .unwrap()
      .into_std()
      .unwrap();

    if let Some(fd) = self.ready_fd {
      assert!(fd >= 3);

      let local_address = listener.local_addr().unwrap();

      let bytes = local_address.port().to_string();

      let result = unsafe { libc::write(fd, bytes.as_ptr().cast(), bytes.len()) };

      assert!(result >= 0);

      assert_eq!(usize::try_from(result).unwrap(), bytes.len());

      let result = unsafe { libc::close(fd) };

      assert_eq!(result, 0);
    }

    axum_server::from_tcp(listener)
      .unwrap()
      .handle(handle)
      .serve(router.into_make_service())
      .await
      .unwrap();

    Ok(())
  }

  async fn get_file(server: Extension<Arc<Server>>, hash: Path<Hash>) -> Vec<u8> {
    server.read_file(*hash).unwrap()
  }

  async fn put_file(server: Extension<Arc<Server>>, hash: Path<Hash>, file: Bytes) {
    server.write_file(*hash, &file).unwrap();
  }
}
