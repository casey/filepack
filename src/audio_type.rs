use super::*;

#[derive(Clone, Copy, Debug, Decode, Display, Encode, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum AudioType {
  #[n(0)]
  #[strum(serialize = "FLAC")]
  Flac,
  #[n(1)]
  #[strum(serialize = "MP3")]
  Mp3,
}

impl ContentType for AudioType {
  const EXTENSIONS: &[&str] = &["flac", "mp3"];

  fn from_extension(extension: &str) -> Option<Self> {
    match extension {
      "flac" => Some(Self::Flac),
      "mp3" => Some(Self::Mp3),
      _ => None,
    }
  }

  fn resource_type(self) -> ResourceType {
    match self {
      Self::Flac => ResourceType::Flac,
      Self::Mp3 => ResourceType::Mp3,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(AudioType::Flac.to_string(), "FLAC");
    assert_eq!(AudioType::Mp3.to_string(), "MP3");
  }

  #[test]
  fn from_path() {
    #[track_caller]
    fn case(path: &str, expected: Result<AudioType, PathError>) {
      assert_eq!(AudioType::from_path(&path.parse().unwrap()), expected);
    }

    case("foo.flac", Ok(AudioType::Flac));
    case("foo.mp3", Ok(AudioType::Mp3));
    case(
      "foo.wav",
      Err(PathError::Extension {
        extensions: &["flac", "mp3"],
      }),
    );
    case(
      "foo",
      Err(PathError::Extension {
        extensions: &["flac", "mp3"],
      }),
    );
  }
}
