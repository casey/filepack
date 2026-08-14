use {super::*, reqwest::blocking::RequestBuilder, url::Host};

pub(crate) struct Client {
  client: reqwest::blocking::Client,
  key: Option<PrivateKey>,
  server: Url,
}

impl Client {
  fn delete(&self, path: &str) -> Result<reqwest::blocking::Response> {
    self
      .request(self.client.delete(self.url(path)))?
      .check_status()
  }

  pub(crate) fn delete_package(&self, fingerprint: Fingerprint) -> Result {
    self.delete(&format!("api/package/{fingerprint}"))?;

    Ok(())
  }

  pub(crate) fn file(&self, hash: Hash) -> Result<reqwest::blocking::Response> {
    self.get(&format!("file/{hash}"))
  }

  pub(crate) fn file_url(&self, hash: Hash) -> Url {
    self.url(&format!("file/{hash}"))
  }

  fn get(&self, path: &str) -> Result<reqwest::blocking::Response> {
    self
      .request(self.client.get(self.url(path)))?
      .check_status()
  }

  pub(crate) fn has_package(&self, fingerprint: Fingerprint) -> Result<bool> {
    self.head(&format!("package/{fingerprint}"))
  }

  fn head(&self, path: &str) -> Result<bool> {
    self.request(self.client.head(self.url(path)))?.found()
  }

  pub(crate) fn missing_files(&self, hashes: BTreeSet<Hash>) -> Result<HashSet<Hash>> {
    let body = api::missing::Request {
      hashes: hashes.into(),
    }
    .encode_to_vec();

    Ok(
      self
        .post_with_body("api/missing", body)?
        .cbor::<api::missing::Response>()?
        .hashes
        .into_iter()
        .collect(),
    )
  }

  pub(crate) fn new(options: &Options, server: Url, auth: Option<&KeyName>) -> Result<Self> {
    install_default_crypto_provider()?;

    let client = reqwest::blocking::Client::builder()
      .connect_timeout(Duration::from_secs(30))
      .http2_adaptive_window(true)
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

  fn post(&self, path: &str) -> Result<reqwest::blocking::Response> {
    self
      .request(self.client.post(self.url(path)))?
      .check_status()
  }

  fn post_with_body(
    &self,
    path: &str,
    body: impl Into<reqwest::blocking::Body>,
  ) -> Result<reqwest::blocking::Response> {
    self
      .request(self.client.post(self.url(path)).body(body.into()))?
      .check_status()
  }

  fn put(
    &self,
    path: &str,
    body: impl Into<reqwest::blocking::Body>,
  ) -> Result<reqwest::blocking::Response> {
    self
      .request(self.client.put(self.url(path)).body(body.into()))?
      .check_status()
  }

  pub(crate) fn put_file(&self, hash: Hash, body: reqwest::blocking::Body) -> Result {
    self.put(&format!("file/{hash}"), body)?;

    Ok(())
  }

  fn request(&self, mut builder: RequestBuilder) -> Result<reqwest::blocking::Response> {
    if let Some(key) = &self.key {
      let host = self.server.host_str().unwrap().to_owned();
      builder = builder.bearer_auth(Token::encode(key, &host)?);
    }

    builder.send().context(error::Request)
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
}
