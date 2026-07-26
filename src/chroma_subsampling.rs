use super::*;

#[derive(Clone, Copy, Debug, Decode, Display, Encode, PartialEq, Serialize)]
pub(crate) enum ChromaSubsampling {
  #[n(0)]
  #[serde(rename = "4:0:0")]
  #[strum(serialize = "4:0:0")]
  Yuv400,
  #[n(1)]
  #[serde(rename = "4:2:0")]
  #[strum(serialize = "4:2:0")]
  Yuv420,
  #[n(2)]
  #[serde(rename = "4:2:2")]
  #[strum(serialize = "4:2:2")]
  Yuv422,
  #[n(3)]
  #[serde(rename = "4:4:0")]
  #[strum(serialize = "4:4:0")]
  Yuv440,
  #[n(4)]
  #[serde(rename = "4:4:4")]
  #[strum(serialize = "4:4:4")]
  Yuv444,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    #[track_caller]
    fn case(subsampling: ChromaSubsampling, expected: &str) {
      assert_eq!(subsampling.to_string(), expected);
    }

    case(ChromaSubsampling::Yuv400, "4:0:0");
    case(ChromaSubsampling::Yuv420, "4:2:0");
    case(ChromaSubsampling::Yuv422, "4:2:2");
    case(ChromaSubsampling::Yuv440, "4:4:0");
    case(ChromaSubsampling::Yuv444, "4:4:4");
  }

  #[test]
  fn encoding() {
    assert_cbor(ChromaSubsampling::Yuv400, "00");
    assert_cbor(ChromaSubsampling::Yuv420, "01");
    assert_cbor(ChromaSubsampling::Yuv422, "02");
    assert_cbor(ChromaSubsampling::Yuv440, "03");
    assert_cbor(ChromaSubsampling::Yuv444, "04");
  }

  #[test]
  fn serialize() {
    #[track_caller]
    fn case(subsampling: ChromaSubsampling, expected: &str) {
      assert_eq!(serde_json::to_string(&subsampling).unwrap(), expected);
    }

    case(ChromaSubsampling::Yuv400, r#""4:0:0""#);
    case(ChromaSubsampling::Yuv420, r#""4:2:0""#);
    case(ChromaSubsampling::Yuv422, r#""4:2:2""#);
    case(ChromaSubsampling::Yuv440, r#""4:4:0""#);
    case(ChromaSubsampling::Yuv444, r#""4:4:4""#);
  }
}
