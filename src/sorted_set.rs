use super::*;

#[derive(Debug, PartialEq)]
pub struct SortedSet<T>(Vec<T>);

impl<T> SortedSet<T> {
  pub fn into_inner(self) -> Vec<T> {
    self.0
  }
}

impl<T: Ord> From<BTreeSet<T>> for SortedSet<T> {
  fn from(set: BTreeSet<T>) -> Self {
    Self(set.into_iter().collect())
  }
}

impl<T> Deref for SortedSet<T> {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T: Encode> Encode for SortedSet<T> {
  fn encode(&self, encoder: &mut Encoder) {
    self.0.encode(encoder);
  }
}

impl<T: Decode + PartialOrd> Decode for SortedSet<T> {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    let elements = Vec::<T>::decode(decoder)?;

    for window in elements.windows(2) {
      ensure!(window[0] < window[1], decode_error::Unsorted);
    }

    Ok(Self(elements))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rejects_duplicate() {
    assert_matches!(
      SortedSet::<u64>::decode_from_slice(&vec![1u64, 1u64].encode_to_vec()),
      Err(DecodeError::Unsorted),
    );
  }

  #[test]
  fn rejects_unsorted() {
    assert_matches!(
      SortedSet::<u64>::decode_from_slice(&vec![2u64, 1u64].encode_to_vec()),
      Err(DecodeError::Unsorted),
    );
  }

  #[test]
  fn round_trip() {
    assert_encoding(SortedSet::<u64>::from(BTreeSet::from([1, 2, 3])));
  }
}
