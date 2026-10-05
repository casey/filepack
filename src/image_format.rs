use super::*;

#[derive(Clone, Copy, Debug, Decode, Display, Encode, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "UPPERCASE")]
pub(crate) enum ImageFormat {
  #[n(0)]
  Jpeg,
  #[n(1)]
  Png,
}

impl ImageFormat {
  pub(crate) fn extension(self) -> &'static str {
    match self {
      Self::Jpeg => "jpg",
      Self::Png => "png",
    }
  }
}

impl Format for ImageFormat {
  const EXTENSIONS: &[&str] = &["jpg", "png"];

  fn from_extension(extension: &str) -> Option<Self> {
    match extension {
      "jpg" => Some(Self::Jpeg),
      "png" => Some(Self::Png),
      _ => None,
    }
  }

  fn resource_type(self) -> ResourceType {
    match self {
      Self::Jpeg => ResourceType::Jpeg,
      Self::Png => ResourceType::Png,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn extension() {
    assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
    assert_eq!(ImageFormat::Png.extension(), "png");
  }

  #[test]
  fn from_path() {
    #[track_caller]
    fn case(path: &str, expected: Result<ImageFormat, PathError>) {
      assert_eq!(ImageFormat::from_path(&path.parse().unwrap()), expected);
    }

    case("foo.jpg", Ok(ImageFormat::Jpeg));
    case("foo.png", Ok(ImageFormat::Png));
    case(
      "foo.svg",
      Err(PathError::Extension {
        extensions: &["jpg", "png"],
      }),
    );
    case(
      "foo",
      Err(PathError::Extension {
        extensions: &["jpg", "png"],
      }),
    );
  }
}
