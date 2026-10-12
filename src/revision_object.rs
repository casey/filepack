use super::*;

#[derive(Debug, Decode, Encode, PartialEq)]
pub struct RevisionObject {
  #[n(1)]
  pub package: Fingerprint,
  #[n(2)]
  pub parents: Option<OrderedSet<Revision>>,
}

impl RevisionObject {
  pub fn hash(&self) -> Revision {
    Hash::bytes(&self.encode_to_vec()).into()
  }

  pub(crate) fn parents(&self) -> &[Revision] {
    self
      .parents
      .as_ref()
      .map(OrderedSet::as_slice)
      .unwrap_or_default()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_encoding(RevisionObject {
      package: Fingerprint::EXAMPLE.parse().unwrap(),
      parents: Some(OrderedSet::singleton(Revision::EXAMPLE.parse().unwrap())),
    });
  }

  #[test]
  fn hash() {
    assert_eq!(
      RevisionObject {
        package: Fingerprint::EXAMPLE.parse().unwrap(),
        parents: None,
      }
      .hash()
      .to_string(),
      Revision::EXAMPLE,
    );
  }
}
