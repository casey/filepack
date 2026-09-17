use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum KeyIdentifier {
  Literal(PublicKey),
  Name(KeyName),
}

impl FromStr for KeyIdentifier {
  type Err = KeyIdentifierError;

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn literal_invalid() {
    let err = "public1zz".parse::<KeyIdentifier>().unwrap_err();
    assert_eq!(err.to_string(), "public key contains invalid hex digit `z`");
    assert_matches!(
      err,
      KeyIdentifierError::PublicKey {
        source: HexError::Digit {
          digit: 'z',
          tag: Tag::PublicKey,
        },
      },
    );
  }

  #[test]
  fn name_invalid() {
    let err = "FOO".parse::<KeyIdentifier>().unwrap_err();
    assert_eq!(err.to_string(), "invalid public key name `FOO`");
    assert_matches!(err, KeyIdentifierError::Name { name } if name == "FOO");
  }
}
