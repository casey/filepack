use super::*;

#[derive(Debug, Decode, Encode)]
#[deco(transparent)]
pub struct InvalidPublicKey(pub(crate) [u8; PublicKey::LEN]);

impl Hex for InvalidPublicKey {
  const EXAMPLE: &'static str =
    "public10000000000000000000000000000000000000000000000000000000000000000";
  const TAG: Tag = Tag::Public;
}

impl Display for InvalidPublicKey {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self.format(f)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(
      InvalidPublicKey([0; PublicKey::LEN]).to_string(),
      InvalidPublicKey::EXAMPLE,
    );

    let public_key = PublicKey::EXAMPLE.parse::<PublicKey>().unwrap();

    assert_eq!(
      InvalidPublicKey(public_key.inner().to_bytes()).to_string(),
      public_key.to_string(),
    );
  }
}
