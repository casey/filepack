use {super::*, reqwest::blocking::RequestBuilder, url::Host};

pub(crate) fn client() -> Result<Client> {
  install_default_crypto_provider()?;

  Client::builder()
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
    .context(error::ClientBuild)
}

pub(crate) fn current_dir() -> Result<Utf8PathBuf> {
  Utf8PathBuf::from_path_buf(env::current_dir().context(error::CurrentDir)?)
    .map_err(|path| error::PathUnicode { path }.build())
}

pub(crate) fn decode_path(path: &Path) -> Result<&Utf8Path> {
  Utf8Path::from_path(path).context(error::PathUnicode { path })
}

pub(crate) fn default<T: Default>() -> T {
  Default::default()
}

pub(crate) fn format_size(size: u64) -> SizeFormatter<u64, FormatSizeOptions> {
  SizeFormatter::new(size, FormatSizeOptions::from(BINARY).decimal_places(1))
}

pub fn install_default_crypto_provider() -> Result {
  static INSTALLED: LazyLock<bool> = LazyLock::new(|| {
    rustls::crypto::ring::default_provider()
      .install_default()
      .is_ok()
  });

  ensure!(*INSTALLED, error::RustlsProvider);

  Ok(())
}

pub(crate) fn is_lowercase_hex(s: &str) -> bool {
  s.chars()
    .all(|c| c.is_ascii_hexdigit() && (c.is_numeric() || c.is_lowercase()))
}

pub(crate) fn load_auth_key(
  options: &Options,
  server: &Url,
  auth: Option<&KeyName>,
) -> Result<Option<PrivateKey>> {
  let Some(name) = auth else {
    return Ok(None);
  };

  let loopback = match server.host().unwrap() {
    Host::Domain(domain) => domain == "localhost",
    Host::Ipv4(addr) => addr.is_loopback(),
    Host::Ipv6(addr) => addr.is_loopback(),
  };

  ensure!(server.scheme() == "https" || loopback, error::TokenOverHttp);

  let keychain = Keychain::load(options)?;

  Ok(Some(PrivateKey::load(
    &keychain.path.join(name.private_key_filename()),
  )?))
}

pub(crate) fn now() -> Result<u64> {
  Ok(
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .context(error::Time)?
      .as_secs(),
  )
}

pub(crate) fn parse_number<T: FromStr<Err = ParseIntError>>(s: &str) -> Result<T, NumberError> {
  ensure! {
    re::NUMBER.is_match(s),
    number_error::Invalid { number: s },
  }

  Ok(s.parse()?)
}

pub(crate) fn request_with_token(
  mut builder: RequestBuilder,
  server: &Url,
  key: Option<&PrivateKey>,
) -> Result<RequestBuilder> {
  if let Some(key) = key {
    let host = server.host_str().unwrap().to_owned();
    builder = builder.bearer_auth(Token::encode(key, &host)?);
  }

  Ok(builder)
}

pub(crate) fn transfer_tempfile(hash: Hash, path: &Utf8Path) -> io::Result<NamedTempFile> {
  tempfile::Builder::new()
    .prefix(&format!("{hash}-"))
    .suffix(".incomplete")
    .tempfile_in(path)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn lowercase_hex() {
    assert!(is_lowercase_hex("0123456789abcdef"));
    assert!(!is_lowercase_hex("0123456789ABCDEF"));
    assert!(!is_lowercase_hex("xyz"));
  }
}
