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
      "8200a1008268666f6f2e666c6163686261722e666c6163",
    );
  }
}
