use {super::*, bech32::Hrp};

pub(crate) enum Bech32m {
  PublicKey,
}

impl Bech32m {
  pub(crate) fn decode(self, s: &str) -> Vec<u8> {
    use bech32::primitives::decode::CheckedHrpstring;
    let p = CheckedHrpstring::new::<bech32::Bech32m>(&s).unwrap();
    assert_eq!(p.hrp(), self.hrp());
    p.byte_iter().collect()
  }

  pub(crate) fn encode(self, f: &mut Formatter, bytes: [u8; PublicKey::LEN]) -> fmt::Result {
    bech32::encode_to_fmt::<bech32::Bech32m, Formatter>(f, self.hrp(), &bytes).map_err(|err| {
      if let bech32::EncodeError::Fmt(err) = err {
        err
      } else {
        unreachable!()
      }
    })
  }

  fn hrp(self) -> Hrp {
    static PUBLIC_KEY_HRP: LazyLock<Hrp> = LazyLock::new(|| Hrp::parse("public").unwrap());

    match self {
      Self::PublicKey => *PUBLIC_KEY_HRP,
    }
  }
}
