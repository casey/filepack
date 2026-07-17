use super::*;

#[derive(Clone, Copy, Debug, Decode, Display, Encode, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "UPPERCASE")]
pub enum AudioCodec {
  #[n(0)]
  Aac,
  #[n(1)]
  Mp3,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor(AudioCodec::Aac, "00");
    assert_cbor(AudioCodec::Mp3, "01");
  }
}
