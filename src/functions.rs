use super::*;

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

pub(crate) fn now() -> Result<u64> {
  Ok(
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .context(error::Time)?
      .as_secs(),
  )
}

pub(crate) fn parse_number<T: FromStr<Err = ParseIntError>>(s: &str) -> Result<T, NumberError> {
  if !re::NUMBER.is_match(s) {
    return Err(number_error::Invalid { number: s }.build());
  }

  Ok(s.parse()?)
}

pub(crate) fn parse_server_url(s: &str) -> Result<Url, String> {
  let url = s.parse::<Url>().map_err(|err| err.to_string())?;

  let scheme = url.scheme();

  if !matches!(scheme, "http" | "https") {
    return Err(format!(
      "URL scheme '{scheme}' not allowed, must be 'http' or 'https'"
    ));
  }

  Ok(url)
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
