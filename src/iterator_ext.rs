use super::*;

pub trait IteratorExt: Iterator {
  fn zip_eq<T>(self, other: T) -> iter::Zip<Self, T::IntoIter>
  where
    Self: ExactSizeIterator + Sized,
    T: IntoIterator<IntoIter: ExactSizeIterator>,
  {
    let other = other.into_iter();

    assert_eq!(self.len(), other.len());

    self.zip(other)
  }
}

impl<T: Iterator> IteratorExt for T {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn zip_eq() {
    assert_eq!(
      [0].into_iter().zip_eq([1]).collect::<Vec<(i32, i32)>>(),
      [(0, 1)],
    );
  }

  #[test]
  #[should_panic(expected = "assertion `left == right` failed\n  left: 1\n right: 2")]
  fn zip_eq_length_mismatch() {
    [0].into_iter().zip_eq([0, 1]).for_each(drop);
  }
}
