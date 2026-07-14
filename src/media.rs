use super::*;

#[derive(
  Clone, Debug, Decode, Deserialize, Encode, EnumDiscriminants, IntoStaticStr, PartialEq, Serialize,
)]
#[serde(deny_unknown_fields, rename_all = "snake_case", tag = "type")]
#[strum(serialize_all = "kebab-case")]
#[strum_discriminants(derive(Display), name(MediaType), strum(serialize_all = "kebab-case"))]
pub(crate) enum Media {
  #[n(0)]
  Audio {
    #[n(0)]
    tracks: Vec<Track>,
  },
  #[n(1)]
  Image {
    #[n(0)]
    images: Vec<Image>,
  },
}

impl Media {
  pub(crate) fn name(&self) -> &'static str {
    self.into()
  }
}

impl MediaType {
  pub(crate) fn noun(self) -> &'static str {
    match self {
      Self::Audio => "track",
      Self::Image => "image",
    }
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
      "8200a10082ab00600160020003000468666f6f2e666c6163050006000760080009000a00ab006001600200030004686261722e666c6163050006000760080009000a00",
    );
  }

  #[test]
  fn image() {
    assert_cbor(
      Media::Image {
        images: vec!["foo.png".parse().unwrap(), "bar.jpg".parse().unwrap()],
      },
      "8201a10082a300a2000001000167666f6f2e706e670201a300a20000010001676261722e6a70670200",
    );
  }
}
