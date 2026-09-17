use super::*;

#[derive(Debug, Decode, Encode)]
#[deco(transparent)]
pub struct InvalidPublicKey(pub(crate) [u8; PublicKey::LEN]);

impl Hex for InvalidPublicKey {
  const TAG: Tag = Tag::PublicKey;
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
      "public10000000000000000000000000000000000000000000000000000000000000000",
    );

    let public_key = test::PUBLIC_KEY.parse::<PublicKey>().unwrap();

    assert_eq!(
      InvalidPublicKey(public_key.inner().to_bytes()).to_string(),
      public_key.to_string(),
    );
  }
}
