use super::*;

/// Tag identifying the type of a tagged hex string.
#[derive(
  Clone,
  Copy,
  Debug,
  Deserialize,
  EnumIter,
  Eq,
  IntoStaticStr,
  Json,
  Ord,
  PartialEq,
  PartialOrd,
  Serialize,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Tag {
  /// BLAKE3 hash of a file or directory.
  Hash,
  /// Fingerprint identifying a package by its content.
  Package,
  /// Private key.
  Private,
  /// Public key.
  Public,
  /// Hash of a revision object.
  Revision,
  /// Signature over a package.
  Signature,
  /// Signed bearer token for server authentication.
  Token,
}

impl Tag {
  pub(crate) fn name(self) -> &'static str {
    match self {
      Self::Hash => "hash",
      Self::Package => "package fingerprint",
      Self::Private => "private key",
      Self::Public => "public key",
      Self::Revision => "revision",
      Self::Signature => "signature",
      Self::Token => "token",
    }
  }

  pub fn prefix(self) -> String {
    format!("{}1", self.tag())
  }

  pub fn tag(self) -> &'static str {
    self.into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn check() {
    for (i, tag) in Tag::iter().enumerate() {
      let tag = tag.tag();

      // lowercase
      assert!(tag.chars().all(|c| c.is_ascii_lowercase()));

      // cannot be confused with hex
      assert!(tag.chars().any(|c| !c.is_ascii_hexdigit()));

      // cannot be confused with reverse hex
      assert!(tag.chars().any(|c| !matches!(c, 'k'..='z')));

      // not a prefix or suffix of another type
      for (j, other) in Tag::iter().enumerate() {
        if i != j {
          assert!(!other.tag().starts_with(tag) && !other.tag().ends_with(tag));
        }
      }
    }
  }
}
