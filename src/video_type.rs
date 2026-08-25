use super::*;

#[derive(Clone, Copy, Debug, Decode, Display, Encode, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "UPPERCASE")]
pub(crate) enum VideoType {
  #[n(0)]
  Mp4,
  #[n(1)]
  #[strum(serialize = "WebM")]
  Webm,
}

impl VideoType {
  const EXTENSIONS: &[&str] = &["mp4", "webm"];

  fn from_extension(extension: &str) -> Option<Self> {
    match extension {
      "mp4" => Some(Self::Mp4),
      "webm" => Some(Self::Webm),
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
      Self::Mp4 => ResourceType::Mp4,
      Self::Webm => ResourceType::Webm,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_path() {
    #[track_caller]
    fn case(path: &str, expected: Result<VideoType, PathError>) {
      assert_eq!(VideoType::from_path(&path.parse().unwrap()), expected);
    }

    case("foo.mp4", Ok(VideoType::Mp4));
    case("foo.webm", Ok(VideoType::Webm));
    case(
      "foo.avi",
      Err(PathError::Extension {
        extensions: &["mp4", "webm"],
      }),
    );
    case(
      "foo",
      Err(PathError::Extension {
        extensions: &["mp4", "webm"],
      }),
    );
  }
}
