use super::*;

#[derive(Clone, Debug, Decode, Encode, PartialEq, Serialize)]
#[deco(non_empty_ordered_set, transparent, validate)]
#[serde(transparent)]
pub struct NonEmptyOrderedSet<T>(Vec<T>);

impl<T> NonEmptyOrderedSet<T> {
  pub(crate) fn as_slice(&self) -> &[T] {
    &self.0
  }

  pub fn singleton(element: T) -> Self {
    Self(vec![element])
  }
}

impl<'de, T: Deserialize<'de> + Ord> Deserialize<'de> for NonEmptyOrderedSet<T> {
  fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    Vec::<T>::deserialize(deserializer)?
      .try_into()
      .map_err(serde::de::Error::custom)
  }
}

impl<T: json::Json> json::Json for NonEmptyOrderedSet<T> {
  fn json_schema(registry: &mut json::Registry) -> String {
    let element = T::json_schema(registry);
    let name = format!("NonEmptyOrderedSet<{element}>");
    json::Schema::NonEmptyOrderedSet { element }.register(&name, registry)
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

  #[test]
  fn serde() {
    let set = NonEmptyOrderedSet::try_from(vec![2u64, 1u64]).unwrap();

    let json = serde_json::to_string(&set).unwrap();

    assert_eq!(json, "[2,1]");

    assert_eq!(
      serde_json::from_str::<NonEmptyOrderedSet<u64>>(&json).unwrap(),
      set,
    );

    assert_eq!(
      serde_json::from_str::<NonEmptyOrderedSet<u64>>("[1,1]")
        .unwrap_err()
        .to_string(),
      "duplicate set element",
    );

    assert_eq!(
      serde_json::from_str::<NonEmptyOrderedSet<u64>>("[]")
        .unwrap_err()
        .to_string(),
      "empty set",
    );
  }
}
