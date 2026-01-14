use super::*;

static NAME_RE: LazyLock<Regex> =
  LazyLock::new(|| Regex::new("^[A-Za-z0-9][A-Za-z0-9._-]*$").unwrap());

static LITERAL_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new("^[A-Za-z0-9]{64}$").unwrap());

#[derive(Clone, Debug)]
pub enum KeyIdentifier {
  Literal(PublicKey),
  Named(String),
}

impl KeyIdentifier {
  pub(crate) fn load(&self, key_dir: &Utf8Path) -> Result<PublicKey> {
    match self {
      Self::Literal(key) => Ok(key.clone()),
      Self::Named(name) => PublicKey::load(&key_dir.join(format!("{name}.public"))),
    }
  }
}

impl FromStr for KeyIdentifier {
  type Err = public_key::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if LITERAL_RE.is_match(s) {
      return Ok(Self::Literal(s.parse()?));
    }

    if !NAME_RE.is_match(s) {
      return Err(public_key::Error::Name { name: s.into() });
    }

    Ok(Self::Named(s.into()))
  }
}

impl Display for KeyIdentifier {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Literal(key) => write!(f, "{key}"),
      Self::Named(name) => write!(f, "{name}"),
    }
  }
}
