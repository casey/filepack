use super::*;

#[derive(Debug, Decode, Encode, PartialEq)]
pub struct RevisionObject {
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
      package: test::FINGERPRINT.parse().unwrap(),
      previous: Some(test::REVISION.parse().unwrap()),
    });
  }

  #[test]
  fn hash() {
    assert_eq!(
      RevisionObject {
        package: test::FINGERPRINT.parse().unwrap(),
        previous: None,
      }
      .hash()
      .to_string(),
      test::REVISION,
    );
  }
}
