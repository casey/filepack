use {super::*, reqwest::blocking::RequestBuilder, url::Host};

pub(crate) struct Client {
  client: reqwest::blocking::Client,
  key: Option<PrivateKey>,
  server: Url,
}

impl Client {
  pub(crate) fn delete_package(&self, fingerprint: Fingerprint) -> Result {
    let request = self
      .client
      .delete(self.url(&format!("api/package/{fingerprint}")));

    self.request(request)?.check_status()?;

    Ok(())
  }

  pub(crate) fn file(&self, hash: Hash) -> Result<reqwest::blocking::Response> {
    let request = self.client.get(self.file_url(hash));

    self.request(request)?.check_status()
  }

  pub(crate) fn file_url(&self, hash: Hash) -> Url {
    self.url(&format!("file/{hash}"))
  }

  pub(crate) fn has_package(&self, fingerprint: Fingerprint) -> Result<bool> {
    let request = self
      .client
      .head(self.url(&format!("package/{fingerprint}")));

    self.request(request)?.found()
  }

  pub(crate) fn missing_files(&self, hashes: BTreeSet<Hash>) -> Result<HashSet<Hash>> {
    let body = api::missing::Request {
      hashes: hashes.into(),
    }
    .encode_to_vec();

    let request = self.client.post(self.url("api/missing")).body(body);

    Ok(
      self
        .request(request)?
        .check_status()?
        .cbor::<api::missing::Response>()?
        .hashes
        .into_inner()
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

  pub(crate) fn put_file(&self, hash: Hash, body: reqwest::blocking::Body) -> Result {
    let request = self.client.put(self.file_url(hash)).body(body);

    self.request(request)?.check_status()?;

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
    let request = self.client.post(self.url(&format!("api/directory/{hash}")));

    self.request(request)?.check_status()?;

    Ok(())
  }

  pub(crate) fn verify_package(&self, fingerprint: Fingerprint) -> Result {
    let request = self
      .client
      .post(self.url(&format!("api/package/{fingerprint}")));

    self.request(request)?.check_status()?;

    Ok(())
  }
}
