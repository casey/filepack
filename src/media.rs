use super::*;

#[derive(Clone, Debug, Deserialize, Encode, Decode, IntoStaticStr, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case", tag = "type")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Media {
  #[n(0)]
  Audio {
    #[n(0)]
    tracks: Vec<filename::Flac>,
  },
}

impl Media {
  pub(crate) fn name(&self) -> &'static str {
    self.into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn audio() {
    assert_cbor(
      Media::Audio {
        tracks: vec!["foo.flac".parse().unwrap(), "bar.flac".parse().unwrap()],
      },
      &[
        0x82, 0x00, 0xa1, 0x00, 0x82, 0x68, 0x66, 0x6f, 0x6f, 0x2e, 0x66, 0x6c, 0x61, 0x63, 0x68,
        0x62, 0x61, 0x72, 0x2e, 0x66, 0x6c, 0x61, 0x63,
      ],
    );
  }

  #[test]
  fn name() {
    assert_eq!(Media::Audio { tracks: Vec::new() }.name(), "audio");
  }
}
