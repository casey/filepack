use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum KeyIdentifier {
  Literal(PublicKey),
  Name(KeyName),
}

impl KeyIdentifier {
  pub(crate) fn load(&self, key_dir: &Utf8Path) -> Result<PublicKey> {
    match self {
      Self::Literal(key) => Ok(key.clone()),
      Self::Name(name) => PublicKey::load(&key_dir.join(name.public_key_filename())),
    }
  }
}

impl FromStr for KeyIdentifier {
  type Err = PublicKeyError;

  fn from_str(name: &str) -> Result<Self, Self::Err> {
    if re::PUBLIC_KEY.is_match(name) {
      return Ok(Self::Literal(name.parse()?));
    }

    Ok(Self::Name(name.parse()?))
  }
}

impl Display for KeyIdentifier {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Literal(key) => write!(f, "{key}"),
      Self::Name(name) => write!(f, "{name}"),
    }
  }
}
