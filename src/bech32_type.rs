use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
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
