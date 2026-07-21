use super::*;

#[derive(Clone, Copy, Debug, Decode, Display, Encode, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "UPPERCASE")]
pub(crate) enum Codec {
  #[n(0)]
  Aac,
  #[n(1)]
  #[strum(serialize = "H.264")]
  H264,
  #[n(2)]
  Mp3,
  #[n(3)]
  #[strum(serialize = "Opus")]
  Opus,
  #[n(4)]
  #[strum(serialize = "Vorbis")]
  Vorbis,
  #[n(5)]
  Vp8,
  #[n(6)]
  Vp9,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    #[track_caller]
    fn case(codec: Codec, expected: &str) {
      assert_eq!(codec.to_string(), expected);
    }

    case(Codec::Aac, "AAC");
    case(Codec::H264, "H.264");
    case(Codec::Mp3, "MP3");
    case(Codec::Opus, "Opus");
    case(Codec::Vorbis, "Vorbis");
    case(Codec::Vp8, "VP8");
    case(Codec::Vp9, "VP9");
  }

  #[test]
  fn encoding() {
    assert_cbor(Codec::Aac, "00");
    assert_cbor(Codec::H264, "01");
    assert_cbor(Codec::Mp3, "02");
    assert_cbor(Codec::Opus, "03");
    assert_cbor(Codec::Vorbis, "04");
    assert_cbor(Codec::Vp8, "05");
    assert_cbor(Codec::Vp9, "06");
  }
}
