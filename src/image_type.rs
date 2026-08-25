use super::*;

#[derive(Clone, Copy, Debug, Decode, Display, Encode, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "UPPERCASE")]
pub(crate) enum ImageType {
  #[n(0)]
  Jpeg,
  #[n(1)]
  Png,
}

impl ImageType {
  const EXTENSIONS: &[&str] = &["jpg", "png"];

  pub(crate) fn extension(self) -> &'static str {
    match self {
      Self::Jpeg => "jpg",
      Self::Png => "png",
    }
  }

  fn from_extension(extension: &str) -> Option<Self> {
    match extension {
      "jpg" => Some(Self::Jpeg),
      "png" => Some(Self::Png),
      _ => None,
    }
  }

  pub(crate) fn from_path(path: &RelativePath) -> Result<Self, PathError> {
    path
      .extension()
      .and_then(Self::from_extension)
      .ok_or(PathError::Extension {
        extensions: Self::EXTENSIONS,
      })
  }

  pub(crate) fn resource_type(self) -> ResourceType {
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
    assert_eq!(ImageType::Jpeg.extension(), "jpg");
    assert_eq!(ImageType::Png.extension(), "png");
  }

  #[test]
  fn from_path() {
    #[track_caller]
    fn case(path: &str, expected: Result<ImageType, PathError>) {
      assert_eq!(ImageType::from_path(&path.parse().unwrap()), expected);
    }

    case("foo.jpg", Ok(ImageType::Jpeg));
    case("foo.png", Ok(ImageType::Png));
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
