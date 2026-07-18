use super::*;

#[skip_serializing_none]
#[derive(Clone, Debug, Decode, Encode, PartialEq, Serialize)]
pub struct Track {
  #[n(0)]
  pub(crate) codec: Codec,
  #[n(1)]
  pub(crate) dimensions: Option<Dimensions>,
}

impl Display for Track {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.codec)?;

    if let Some(dimensions) = self.dimensions {
      write!(f, " {dimensions}")?;
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    #[track_caller]
    fn case(track: Track, expected: &str) {
      assert_eq!(track.to_string(), expected);
    }

    case(
      Track {
        codec: Codec::Aac,
        dimensions: None,
      },
      "AAC",
    );

    case(
      Track {
        codec: Codec::H264,
        dimensions: Some(Dimensions {
          height: 1,
          width: 2,
        }),
      },
      "H264 2×1",
    );
  }

  #[test]
  fn encoding() {
    assert_cbor(
      Track {
        codec: Codec::Aac,
        dimensions: None,
      },
      "a10000",
    );

    assert_cbor(
      Track {
        codec: Codec::H264,
        dimensions: Some(Dimensions {
          height: 1,
          width: 2,
        }),
      },
      "a2000101a200010102",
    );
  }

  #[test]
  fn serialize() {
    assert_eq!(
      serde_json::to_string(&Track {
        codec: Codec::Aac,
        dimensions: None,
      })
      .unwrap(),
      r#"{"codec":"aac"}"#,
    );

    assert_eq!(
      serde_json::to_string(&Track {
        codec: Codec::H264,
        dimensions: Some(Dimensions {
          height: 1,
          width: 2,
        }),
      })
      .unwrap(),
      r#"{"codec":"h264","dimensions":{"height":1,"width":2}}"#,
    );
  }
}
