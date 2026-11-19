use super::*;

pub(crate) struct StubServer {
  listener: TcpListener,
  responses: BTreeMap<String, Vec<u8>>,
  stop: Arc<AtomicBool>,
  thread: Option<thread::JoinHandle<()>>,
}

impl StubServer {
  pub(crate) fn address(&self) -> String {
    format!("http://{}", self.listener.local_addr().unwrap())
  }

  pub(crate) fn file(mut self, content: impl Into<Vec<u8>>) -> Self {
    let content = content.into();

    self
      .responses
      .insert(format!("/api/file/{}", Hash::bytes(&content)), content);

    self
  }

  pub(crate) fn new() -> Self {
    Self {
      listener: TcpListener::bind("127.0.0.1:0").unwrap(),
      responses: BTreeMap::new(),
      stop: Arc::new(AtomicBool::new(false)),
      thread: None,
    }
  }

  pub(crate) fn revision(mut self, revision: &Revision, signatures: &[Hash]) -> Self {
    self.responses.insert(
      format!("/api/revision/{}", revision.hash()),
      api::revision::get::Response {
        numbers: BTreeSet::new().into(),
        signatures: signatures
          .iter()
          .copied()
          .collect::<BTreeSet<Hash>>()
          .into(),
      }
      .encode_to_vec(),
    );

    self
  }

  pub(crate) fn spawn(mut self) -> Self {
    let listener = self.listener.try_clone().unwrap();

    let responses = self.responses.clone();

    let stop = self.stop.clone();

    self.thread = Some(thread::spawn(move || {
      for stream in listener.incoming() {
        if stop.load(Ordering::Relaxed) {
          break;
        }

        let mut stream = stream.unwrap();

        let request = BufReader::new(&stream)
          .lines()
          .map(Result::unwrap)
          .take_while(|line| !line.is_empty())
          .collect::<Vec<String>>();

        let Some(line) = request.first() else {
          continue;
        };

        let path = line.split(' ').nth(1).unwrap();

        if let Some(body) = responses.get(path) {
          write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len(),
          )
          .unwrap();

          stream.write_all(body).unwrap();
        } else {
          stream
            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .unwrap();
        }
      }
    }));

    self
  }
}

impl Drop for StubServer {
  fn drop(&mut self) {
    if let Some(thread) = self.thread.take() {
      self.stop.store(true, Ordering::Relaxed);

      TcpStream::connect(self.listener.local_addr().unwrap()).unwrap();

      thread.join().unwrap();
    }
  }
}
