use super::*;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Fingerprint(pub(crate) Hash);

impl Fingerprint {
  pub(crate) const LEN: usize = Hash::LEN;

  #[must_use]
  pub(crate) fn as_bytes(&self) -> &[u8; Self::LEN] {
    self.0.as_bytes()
  }
}

impl Display for Fingerprint {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl FromStr for Fingerprint {
  type Err = HashError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self(s.parse()?))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn uppercase_is_forbidden() {
    test::HASH
      .to_uppercase()
      .parse::<Fingerprint>()
      .unwrap_err();
  }
}
