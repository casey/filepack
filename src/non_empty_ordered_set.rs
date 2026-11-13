use super::*;

#[derive(Debug, Decode, Encode, PartialEq)]
#[deco(non_empty_ordered_set, transparent, validate)]
pub struct NonEmptyOrderedSet<T>(Vec<T>);

impl<T> NonEmptyOrderedSet<T> {
  pub(crate) fn as_slice(&self) -> &[T] {
    &self.0
  }

  pub fn singleton(element: T) -> Self {
    Self(vec![element])
  }
}

impl<T: Ord> TryFrom<Vec<T>> for NonEmptyOrderedSet<T> {
  type Error = MalformedError;

  fn try_from(elements: Vec<T>) -> Result<Self, Self::Error> {
    let set = Self(elements);
    set.validate()?;
    Ok(set)
  }
}

impl<T: Ord> Validate for NonEmptyOrderedSet<T> {
  fn validate(&self) -> Result<(), MalformedError> {
    ensure!(!self.0.is_empty(), malformed_error::EmptySet);

    let mut seen = BTreeSet::new();

    for element in &self.0 {
      ensure!(seen.insert(element), malformed_error::DuplicateElement);
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rejects_duplicate() {
    assert_matches!(
      NonEmptyOrderedSet::<u64>::decode_strict(&vec![1u64, 1u64].encode_to_vec()),
      Err(DecodeError::Malformed(MalformedError::DuplicateElement)),
    );

    assert_matches!(
      NonEmptyOrderedSet::try_from(vec![1u64, 1u64]),
      Err(MalformedError::DuplicateElement),
    );
  }

  #[test]
  fn rejects_empty() {
    assert_matches!(
      NonEmptyOrderedSet::<u64>::decode_strict(&Vec::<u64>::new().encode_to_vec()),
      Err(DecodeError::Malformed(MalformedError::EmptySet)),
    );

    assert_matches!(
      NonEmptyOrderedSet::try_from(Vec::<u64>::new()),
      Err(MalformedError::EmptySet),
    );
  }

  #[test]
  fn round_trip() {
    assert_encoding(NonEmptyOrderedSet::try_from(vec![2u64, 1u64]).unwrap());
  }
}
