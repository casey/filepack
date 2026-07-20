use super::*;

#[derive(Clone, Copy, Debug, Decode, Display, Encode, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "UPPERCASE")]
pub(crate) enum Codec {
  #[n(0)]
  Aac,
  #[n(1)]
  H264,
  #[n(2)]
  Mp3,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor(Codec::Aac, "00");
    assert_cbor(Codec::H264, "01");
    assert_cbor(Codec::Mp3, "02");
  }
}
