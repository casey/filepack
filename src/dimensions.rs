use super::*;

#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, Serialize)]
pub struct Dimensions {
  #[n(0)]
  pub(crate) height: u64,
  #[n(1)]
  pub(crate) width: u64,
}

impl Display for Dimensions {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}×{}", self.width, self.height)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor(
      Dimensions {
        height: 1,
        width: 2,
      },
      "a200010102",
    );
  }
}
