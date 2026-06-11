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
    tracks: Vec<filename::Flac>,
  },
  #[n(1)]
  Image {
    #[n(0)]
    images: Vec<filename::Image>,
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
      "8200a1008268666f6f2e666c6163686261722e666c6163",
    );
  }

  #[test]
  fn image() {
    assert_cbor(
      Media::Image {
        images: vec!["foo.png".parse().unwrap(), "bar.jpg".parse().unwrap()],
      },
      "8201a1008267666f6f2e706e67676261722e6a7067",
    );
  }
}
