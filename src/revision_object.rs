use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Debug, Decode, Encode, PartialEq)]
pub struct RevisionObject {
  #[n(0)]
  pub version: Version,
  #[n(1)]
  pub package: Fingerprint,
  #[n(2)]
  pub previous: Option<Revision>,
}

impl RevisionObject {
  pub fn hash(&self) -> Revision {
    Hash::bytes(&self.encode_to_vec()).into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_encoding(RevisionObject {
      version: Version::Zero,
      package: test::FINGERPRINT.parse().unwrap(),
      previous: Some(test::REVISION.parse().unwrap()),
    });
  }

  #[test]
  fn hash() {
    assert_eq!(
      RevisionObject {
        version: Version::Zero,
        package: test::FINGERPRINT.parse().unwrap(),
        previous: None,
      }
      .hash()
      .to_string(),
      test::REVISION,
    );
  }
}
