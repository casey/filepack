use super::*;

#[derive(Clone, Copy, Debug, EnumIter, PartialEq)]
pub enum Tag {
  Fingerprint,
  Hash,
  PrivateKey,
  PublicKey,
  Signature,
  Token,
}

impl Tag {
  pub(crate) fn name(self) -> &'static str {
    match self {
      Self::Fingerprint => "package fingerprint",
      Self::Hash => "hash",
      Self::PrivateKey => "private key",
      Self::PublicKey => "public key",
      Self::Signature => "signature",
      Self::Token => "token",
    }
  }

  pub(crate) fn prefix(self) -> &'static str {
    match self {
      Self::Fingerprint => "package",
      Self::Hash => "hash",
      Self::PrivateKey => "private",
      Self::PublicKey => "public",
      Self::Signature => "signature",
      Self::Token => "token",
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn check() {
    for (i, tag) in Tag::iter().enumerate() {
      let prefix = tag.prefix();

      // lowercase
      assert!(prefix.chars().all(|c| c.is_ascii_lowercase()));

      // cannot be confused with hex
      assert!(prefix.chars().any(|c| !c.is_ascii_hexdigit()));

      // cannot be confused with reverse hex
      assert!(prefix.chars().any(|c| !matches!(c, 'k'..='z')));

      // not a prefix or suffix of another type
      assert!(Tag::iter().enumerate().all(|(j, other)| j == i
        || (!other.prefix().starts_with(prefix) && !other.prefix().ends_with(prefix))));
    }
  }
}
