use super::*;

#[derive(Clone, Copy, Debug, Decode, Display, Encode, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "UPPERCASE")]
pub enum VideoCodec {
  #[n(0)]
  H263,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor(VideoCodec::H263, "00");
  }
}
