use super::*;

#[derive(Debug, Decode, Encode, PartialEq)]
#[deco(transparent, validate)]
pub struct OrderedSet<T>(Vec<T>);

impl<T> OrderedSet<T> {
  pub fn as_slice(&self) -> &[T] {
    &self.0
  }

  pub fn singleton(element: T) -> Self {
    Self(vec![element])
  }
}

impl<T> Deref for OrderedSet<T> {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T> IntoIterator for OrderedSet<T> {
  type IntoIter = vec::IntoIter<T>;
  type Item = T;

  fn into_iter(self) -> Self::IntoIter {
    self.0.into_iter()
  }
}

impl<'a, T> IntoIterator for &'a OrderedSet<T> {
  type IntoIter = slice::Iter<'a, T>;
  type Item = &'a T;

  fn into_iter(self) -> Self::IntoIter {
    self.0.iter()
  }
}

impl<T: Ord> TryFrom<Vec<T>> for OrderedSet<T> {
  type Error = MalformedError;

  fn try_from(elements: Vec<T>) -> Result<Self, Self::Error> {
    let set = Self(elements);
    set.validate()?;
    Ok(set)
  }
}

impl<T: Ord> Validate for OrderedSet<T> {
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
      OrderedSet::<u64>::decode_strict(&vec![1u64, 1u64].encode_to_vec()),
      Err(DecodeError::Malformed(MalformedError::DuplicateElement)),
    );

    assert_matches!(
      OrderedSet::try_from(vec![1u64, 1u64]),
      Err(MalformedError::DuplicateElement),
    );
  }

  #[test]
  fn rejects_empty() {
    assert_matches!(
      OrderedSet::<u64>::decode_strict(&Vec::<u64>::new().encode_to_vec()),
      Err(DecodeError::Malformed(MalformedError::EmptySet)),
    );

    assert_matches!(
      OrderedSet::try_from(Vec::<u64>::new()),
      Err(MalformedError::EmptySet),
    );
  }

  #[test]
  fn round_trip() {
    assert_encoding(OrderedSet::try_from(vec![2u64, 1u64]).unwrap());
  }
}
