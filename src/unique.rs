use super::*;

#[derive(Debug, PartialEq)]
pub struct Unique<T>(Vec<T>);

impl<T> Unique<T> {
  pub fn into_inner(self) -> Vec<T> {
    self.0
  }
}

impl<T: Ord> From<BTreeSet<T>> for Unique<T> {
  fn from(set: BTreeSet<T>) -> Self {
    Self(set.into_iter().collect())
  }
}

impl<T> Deref for Unique<T> {
  type Target = [T];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl<T: Encode> Encode for Unique<T> {
  fn encode(&self, encoder: &mut Encoder) {
    self.0.encode(encoder);
  }
}

impl<T: Decode + PartialOrd> Decode for Unique<T> {
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
      Unique::<u64>::decode_from_slice(&vec![1u64, 1u64].encode_to_vec()),
      Err(DecodeError::Unsorted),
    );
  }

  #[test]
  fn rejects_unsorted() {
    assert_matches!(
      Unique::<u64>::decode_from_slice(&vec![2u64, 1u64].encode_to_vec()),
      Err(DecodeError::Unsorted),
    );
  }

  #[test]
  fn round_trip() {
    assert_encoding(Unique::<u64>::from(BTreeSet::from([1, 2, 3])));
  }
}
