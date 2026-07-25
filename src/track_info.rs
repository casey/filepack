use super::*;

#[skip_serializing_none]
#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub(crate) enum TrackInfo {
  #[n(0)]
  Audio,
  #[n(1)]
  Video {
    #[n(0)]
    bit_depth: Option<u64>,
    #[n(1)]
    dimensions: Dimensions,
    #[n(2)]
    frames: u64,
  },
}

impl TrackInfo {
  pub(crate) fn properties(&self) -> Value {
    match self {
      Self::Audio => Value::Group(vec![("type".into(), Value::scalar("audio"))]),
      Self::Video {
        bit_depth,
        dimensions,
        frames,
      } => {
        let mut properties = Vec::new();

        if let Some(bit_depth) = bit_depth {
          properties.push(("bit depth".into(), Value::scalar(bit_depth)));
        }

        properties.push(("dimensions".into(), Value::scalar(dimensions)));
        properties.push(("frames".into(), Value::scalar(frames)));
        properties.push(("type".into(), Value::scalar("video")));

        Value::Group(properties)
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor(TrackInfo::Audio, "00");

    assert_cbor(
      TrackInfo::Video {
        bit_depth: Some(8),
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 0,
      },
      "8201a3000801a2000101020200",
    );

    assert_cbor(
      TrackInfo::Video {
        bit_depth: None,
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 0,
      },
      "8201a201a2000101020200",
    );
  }

  #[test]
  fn properties() {
    #[track_caller]
    fn case(info: TrackInfo, expected: Value) {
      assert_eq!(info.properties(), expected);
    }

    case(
      TrackInfo::Audio,
      Value::Group(vec![("type".into(), Value::scalar("audio"))]),
    );

    case(
      TrackInfo::Video {
        bit_depth: None,
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 3,
      },
      Value::Group(vec![
        ("dimensions".into(), Value::scalar("2×1")),
        ("frames".into(), Value::scalar(3)),
        ("type".into(), Value::scalar("video")),
      ]),
    );

    case(
      TrackInfo::Video {
        bit_depth: Some(8),
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        frames: 3,
      },
      Value::Group(vec![
        ("bit depth".into(), Value::scalar(8)),
        ("dimensions".into(), Value::scalar("2×1")),
        ("frames".into(), Value::scalar(3)),
        ("type".into(), Value::scalar("video")),
      ]),
    );
  }
}
