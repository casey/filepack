use super::*;

/// Revision of a package.
#[derive(Debug, Decode, Encode, PartialEq)]
pub struct Revision {
  /// Fingerprint at this revision.
  #[n(1)]
  pub package: Fingerprint,
  /// Hashes of the revisions that this revision was derived from.
  #[n(2)]
  pub parents: Option<NonEmptyOrderedSet<RevisionHash>>,
}

impl Revision {
  pub fn hash(&self) -> RevisionHash {
    Hash::bytes(&self.encode_to_vec()).into()
  }

  pub(crate) fn parents(&self) -> &[RevisionHash] {
    self
      .parents
      .as_ref()
      .map_or_default(NonEmptyOrderedSet::as_slice)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_encoding(Revision {
      package: Fingerprint::EXAMPLE.parse().unwrap(),
      parents: Some(NonEmptyOrderedSet::singleton(
        RevisionHash::EXAMPLE.parse().unwrap(),
      )),
    });
  }

  #[test]
  fn hash() {
    assert_eq!(
      Revision {
        package: Fingerprint::EXAMPLE.parse().unwrap(),
        parents: None,
      }
      .hash()
      .to_string(),
      RevisionHash::EXAMPLE,
    );
  }
}
