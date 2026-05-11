use super::*;

// todo:
// - write an error string to ready_fd if we fail to start

static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

#[derive(Parser)]
pub(crate) struct Node {
  address: String,
  #[arg(long)]
  ready_fd: Option<std::os::fd::RawFd>,
}

impl Node {
  pub(crate) fn run(self, options: Options) -> Result {
    let files = options.data_dir()?.join("files");

    ctrlc::set_handler(move || {
      if SHUTTING_DOWN.fetch_or(true, atomic::Ordering::Relaxed) {
        process::exit(1);
      }
    })
    .expect("failed to set <CTRL-C> handler");

    filesystem::create_dir_all(&files)?;

    let listener = TcpListener::bind(self.address).unwrap();

    listener.set_nonblocking(true).unwrap();

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

    while !SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
      match listener.accept() {
        Ok((mut stream, _addr)) => {
          let message = Message::read(&mut stream);

          match message {
            Message::Download(download) => {
              let path = files.join(download.hash.to_string());
              let file = filesystem::read(&path)?;
              Message::File(message::File { file }).write(&mut stream);
              stream.shutdown(net::Shutdown::Both).unwrap();
            }
            Message::Upload(upload) => {
              let actual = Hash::bytes(&upload.file);
              assert_eq!(actual, upload.hash);
              let path = files.join(actual.to_string());
              // don't write if it already exists (use create options)
              filesystem::write(&path, upload.file)?;
              Message::Ok.write(&mut stream);
              stream.shutdown(net::Shutdown::Both).unwrap();
            }
            Message::File(_) | Message::Ok => todo!(),
          }
        }
        Err(err) if err.kind() == io::ErrorKind::WouldBlock => {
          std::thread::sleep(std::time::Duration::from_millis(10));
        }
        Err(err) => panic!("{}", err),
      }
    }

    Ok(())
  }
}
