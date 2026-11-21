use {
  super::*,
  reqwest::{blocking::Body, redirect},
  url::Host,
};

pub(crate) struct Client {
  client: reqwest::blocking::Client,
  key: Option<PrivateKey>,
  server: ServerUrl,
}

impl Client {
  fn delete(&self, path: &str) -> Result<reqwest::blocking::Response> {
    self.request(Method::DELETE, path, None)?.check_status()
  }

  pub(crate) fn delete_number(&self, number: u64) -> Result {
    self.delete(&format!("api/number/{number}"))?;

    Ok(())
  }

  pub(crate) fn file(&self, hash: Hash) -> Result<reqwest::blocking::Response> {
    self.get(&format!("api/file/{hash}"))
  }

  pub(crate) fn file_bytes(&self, hash: Hash) -> Result<Vec<u8>> {
    let bytes = self
      .file(hash)?
      .bytes()
      .with_context(|_| error::ResponseBody {
        url: self.file_url(hash),
      })?;

    let actual = Hash::bytes(&bytes);

    ensure! {
      actual == hash,
      error::DownloadHashMismatch { actual, expected: hash },
    }

    Ok(bytes.into())
  }

  pub(crate) fn file_url(&self, hash: Hash) -> Url {
    self.url(&format!("api/file/{hash}"))
  }

  pub(crate) fn gc(&self) -> Result<api::gc::Response> {
    self.post("api/gc")?.deco()
  }

  fn get(&self, path: &str) -> Result<reqwest::blocking::Response> {
    self.request(Method::GET, path, None)?.check_status()
  }

  pub(crate) fn missing_files(&self, hashes: BTreeSet<Hash>) -> Result<HashSet<Hash>> {
    Ok(
      self
        .post_with_body(
          "api/missing",
          api::missing::Request {
            hashes: hashes.into(),
          },
        )?
        .deco::<api::missing::Response>()?
        .hashes
        .into_iter()
        .collect(),
    )
  }

  pub(crate) fn new(options: &Options, server: ServerUrl, auth: Option<&KeyName>) -> Result<Self> {
    install_default_crypto_provider()?;

    let client = reqwest::blocking::Client::builder()
      .connect_timeout(Duration::from_secs(30))
      .http2_adaptive_window(true)
      .redirect(redirect::Policy::none())
      .tcp_keepalive(Duration::from_secs(30))
      .timeout(None::<Duration>)
      .user_agent(concat!(
        env!("CARGO_PKG_NAME"),
        "/",
        env!("CARGO_PKG_VERSION")
      ))
      .build()
      .context(error::ClientBuild)?;

    let key = if let Some(name) = auth {
      let loopback = match server.host().unwrap() {
        Host::Domain(domain) => domain == "localhost",
        Host::Ipv4(addr) => addr.is_loopback(),
        Host::Ipv6(addr) => addr.is_loopback(),
      };

      ensure!(server.scheme() == "https" || loopback, error::TokenOverHttp);

      let keychain = Keychain::load(options)?;

      Some(PrivateKey::load(
        &keychain.path.join(name.private_key_filename()),
      )?)
    } else {
      None
    };

    Ok(Self {
      client,
      key,
      server,
    })
  }

  pub(crate) fn number(&self, number: u64) -> Result<api::number::Response> {
    self.get(&format!("api/number/{number}"))?.deco()
  }

  pub(crate) fn numbers(&self) -> Result<SortedSet<u64>> {
    Ok(
      self
        .get("api/numbers")?
        .deco::<api::numbers::Response>()?
        .numbers,
    )
  }

  pub(crate) fn package_url(&self, fingerprint: Fingerprint) -> Url {
    self.url(&format!("package/{fingerprint}"))
  }

  fn post(&self, path: &str) -> Result<reqwest::blocking::Response> {
    self.request(Method::POST, path, None)?.check_status()
  }

  fn post_with_body(&self, path: &str, body: impl Encode) -> Result<reqwest::blocking::Response> {
    self
      .request(Method::POST, path, Some(body.encode_to_vec().into()))?
      .check_status()
  }

  fn put(&self, path: &str, body: impl Into<Body>) -> Result<reqwest::blocking::Response> {
    self
      .request(Method::PUT, path, Some(body.into()))?
      .check_status()
  }

  pub(crate) fn put_file(&self, hash: Hash, body: Body) -> Result {
    self.put(&format!("api/file/{hash}"), body)?;
    Ok(())
  }

  fn request(
    &self,
    method: Method,
    path: &str,
    body: Option<Body>,
  ) -> Result<reqwest::blocking::Response> {
    let mut builder = self.client.request(method.clone(), self.url(path));

    if let Some(body) = body {
      builder = builder.body(body);
    }

    if let Some(key) = &self.key
      && !method.is_safe()
    {
      builder = builder.bearer_auth(Claims::sign(
        key,
        self.server.host_str().unwrap(),
        &method,
        &format!("/{path}"),
      )?);
    }

    builder.send().context(error::Request)
  }

  pub(crate) fn revision(&self, revision: RevisionHash) -> Result<api::revision::get::Response> {
    self.get(&format!("api/revision/{revision}"))?.deco()
  }

  pub(crate) fn revision_opt(
    &self,
    revision: RevisionHash,
  ) -> Result<Option<api::revision::get::Response>> {
    self
      .request(Method::GET, &format!("api/revision/{revision}"), None)?
      .found()?
      .map(ReqwestResponseExt::deco)
      .transpose()
  }

  fn url(&self, path: &str) -> Url {
    self.server.join(path).unwrap()
  }

  pub(crate) fn verify_directory(&self, hash: Hash) -> Result {
    self.post(&format!("api/directory/{hash}"))?;
    Ok(())
  }

  pub(crate) fn verify_package(&self, fingerprint: Fingerprint) -> Result {
    self.post(&format!("api/package/{fingerprint}"))?;
    Ok(())
  }

  pub(crate) fn verify_revision(
    &self,
    revision: RevisionHash,
    request: api::revision::post::Request,
  ) -> Result<u64> {
    Ok(
      self
        .post_with_body(&format!("api/revision/{revision}"), request)?
        .deco::<api::revision::post::Response>()?
        .number,
    )
  }

  pub(crate) fn verify_signature(&self, hash: Hash) -> Result {
    self.post(&format!("api/signature/{hash}"))?;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use {super::*, std::io::BufRead, std::net::TcpListener, std::thread};

  #[test]
  fn redirects_are_not_followed() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    thread::spawn(move || {
      let (mut stream, _) = listener.accept().unwrap();
      let mut lines = BufReader::new(&stream).lines();
      while !lines.next().unwrap().unwrap().is_empty() {}
      stream
        .write_all(
          b"HTTP/1.1 308 Permanent Redirect\r\n\
            Location: http://127.0.0.1:1/api/numbers\r\n\
            Content-Length: 0\r\n\
            \r\n",
        )
        .unwrap();
    });

    let client = Client::new(
      &Options::parse_from(["filepack"]),
      format!("http://127.0.0.1:{port}").parse().unwrap(),
      None,
    )
    .unwrap();

    assert_eq!(
      client.numbers().unwrap_err().to_string(),
      format!(
        "response from http://127.0.0.1:{port}/api/numbers failed with status 308 Permanent Redirect: "
      ),
    );
  }

  #[test]
  fn tokens_are_only_sent_with_unsafe_methods() {
    let (_tempdir, data_dir) = tempdir();

    let options = Options::parse_from(["filepack", "--data-dir", data_dir.as_str()]);

    Keychain::load(&options)
      .unwrap()
      .generate_key(&KeyName::DEFAULT)
      .unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = thread::spawn(move || {
      (0..2)
        .map(|_| {
          let (mut stream, _) = listener.accept().unwrap();

          let request = BufReader::new(&stream)
            .lines()
            .map(Result::unwrap)
            .take_while(|line| !line.is_empty())
            .collect::<Vec<String>>();

          stream
            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .unwrap();

          (
            request[0].clone(),
            request
              .iter()
              .any(|line| line.to_lowercase().starts_with("authorization:")),
          )
        })
        .collect::<Vec<(String, bool)>>()
    });

    let client = Client::new(
      &options,
      format!("http://127.0.0.1:{port}").parse().unwrap(),
      Some(&KeyName::DEFAULT),
    )
    .unwrap();

    client.numbers().unwrap_err();

    client.delete_number(0).unwrap_err();

    assert_eq!(
      server.join().unwrap(),
      [
        ("GET /api/numbers HTTP/1.1".into(), false),
        ("DELETE /api/number/0 HTTP/1.1".into(), true),
      ],
    );
  }
}
