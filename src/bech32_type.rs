use super::*;

#[derive(Clone, Copy, Debug, EnumIter, PartialEq)]
pub enum Bech32Type {
  Fingerprint,
  PrivateKey,
  PublicKey,
  Signature,
}

impl Bech32Type {
  pub(crate) fn hrp(self) -> &'static Hrp {
    static FINGERPRINT: Hrp = Hrp::parse_unchecked("package");
    static PRIVATE_KEY: Hrp = Hrp::parse_unchecked("private");
    static PUBLIC_KEY: Hrp = Hrp::parse_unchecked("public");
    static SIGNATURE: Hrp = Hrp::parse_unchecked("signature");

    match self {
      Self::Fingerprint => &FINGERPRINT,
      Self::PrivateKey => &PRIVATE_KEY,
      Self::PublicKey => &PUBLIC_KEY,
      Self::Signature => &SIGNATURE,
    }
  }
}

impl Display for Bech32Type {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Fingerprint => write!(f, "package fingerprint"),
      Self::PrivateKey => write!(f, "private key"),
      Self::PublicKey => write!(f, "public key"),
      Self::Signature => write!(f, "signature"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn check() {
    for (i, ty) in Bech32Type::iter().enumerate() {
      let hrp = ty.hrp().as_str();

      // valid
      Hrp::parse(hrp).unwrap();

      // lowercase
      assert!(hrp.chars().all(|c| c.is_ascii_lowercase()));

      // cannot be confused with hex
      assert!(hrp.chars().any(|c| !c.is_ascii_hexdigit()));

      // cannot be confused with reverse hex
      assert!(hrp.chars().any(|c| !matches!(c, 'k'..='z')));

      // not a prefix or suffix of another type
      assert!(Bech32Type::iter().enumerate().all(|(j, other)| j == i
        || (!other.hrp().as_str().starts_with(hrp) && !other.hrp().as_str().ends_with(hrp))));
    }
  }
}
