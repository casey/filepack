use super::*;

pub(crate) use self::error::Error;

mod error;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) struct RelativePath(String);

impl RelativePath {
  const SEPARATORS: [char; 2] = ['/', '\\'];

  const WINDOWS_RESERVED_CHARACTERS: [char; 7] = ['"', '*', ':', '<', '>', '?', '|'];

  const WINDOWS_RESERVED_NAMES: [&'static str; 28] = [
    "AUX", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "COM¹", "COM²",
    "COM³", "CON", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9", "LPT¹",
    "LPT²", "LPT³", "NUL", "PRN",
  ];

  pub(crate) fn check_portability(&self) -> Option<Lint> {
    for component in Utf8Path::new(&self.0).components() {
      let Utf8Component::Normal(component) = component else {
        unreachable!(
          "it is not possible to construct a `RelativePath` containing non-normal components",
        );
      };

      for character in component.chars() {
        if Self::WINDOWS_RESERVED_CHARACTERS.contains(&character) {
          return Some(Lint::WindowsReservedCharacter { character });
        }

        if let 0..32 = character as u32 {
          return Some(Lint::WindowsReservedCharacter { character });
        }
      }

      for name in Self::WINDOWS_RESERVED_NAMES {
        let uppercase = component.to_uppercase();

        if uppercase == name {
          return Some(Lint::WindowsReservedFilename {
            name: component.into(),
          });
        }

        if uppercase.starts_with(name) && uppercase.chars().nth(name.chars().count()) == Some('.') {
          return Some(Lint::WindowsReservedFilename {
            name: component.into(),
          });
        }
      }

      if component.starts_with(' ') {
        return Some(Lint::WindowsLeadingSpace);
      }

      if component.ends_with(' ') {
        return Some(Lint::WindowsTrailingSpace);
      }

      if component.ends_with('.') {
        return Some(Lint::WindowsTrailingPeriod);
      }

      if component.len() > 255 {
        return Some(Lint::FilenameLength);
      }
    }

    None
  }

  pub(crate) fn to_lowercase(&self) -> Self {
    Self(self.0.to_lowercase())
  }
}

impl AsRef<str> for RelativePath {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl AsRef<Utf8Path> for RelativePath {
  fn as_ref(&self) -> &Utf8Path {
    self.0.as_ref()
  }
}

impl<'de> Deserialize<'de> for RelativePath {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    use serde::de::{Error, Unexpected};

    let s = String::deserialize(deserializer)?;

    s.parse::<Self>()
      .map_err(|err| D::Error::invalid_value(Unexpected::Str(&s), &err.to_string().as_str()))
  }
}

impl Display for RelativePath {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl From<&RelativePath> for RelativePath {
  fn from(path: &RelativePath) -> Self {
    path.clone()
  }
}

impl FromStr for RelativePath {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.starts_with('/') {
      return Err(Error::LeadingSlash);
    }

    if s.ends_with('/') {
      return Err(Error::TrailingSlash);
    }

    if s.contains("//") {
      return Err(Error::DoubleSlash);
    }

    let mut chars = s.chars();
    let first = chars.next();
    let second = chars.next();
    if let Some((first, second)) = first.zip(second) {
      if second == ':' {
        return Err(Error::WindowsDiskPrefix { letter: first });
      }
    }

    let mut path = String::new();

    for (i, component) in s.split('/').enumerate() {
      if component == ".." || component == "." {
        return Err(Error::Component {
          component: component.into(),
        });
      }

      for character in component.chars() {
        if Self::SEPARATORS.contains(&character) {
          return Err(Error::Separator { character });
        }
      }

      if i > 0 {
        path.push('/');
      }

      path.push_str(component);
    }

    if path.is_empty() {
      return Err(Error::Empty);
    }

    Ok(Self(path))
  }
}

impl PartialEq<&str> for RelativePath {
  fn eq(&self, other: &&str) -> bool {
    self.0 == *other
  }
}

impl TryFrom<&Utf8Path> for RelativePath {
  type Error = Error;

  fn try_from(path: &Utf8Path) -> Result<Self, Self::Error> {
    let mut s = String::new();

    for (i, component) in path.components().enumerate() {
      let Utf8Component::Normal(component) = component else {
        return Err(Error::Component {
          component: component.to_string(),
        });
      };

      if i > 0 {
        s.push('/');
      }

      s.push_str(component);
    }

    s.parse()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_str() {
    #[track_caller]
    fn case(path: &str, expected: &str) {
      assert_eq!(path.parse::<RelativePath>().unwrap(), expected);
    }

    case("foo", "foo");
    case("foo/bar", "foo/bar");
  }

  #[test]
  fn from_str_errors() {
    #[track_caller]
    fn case(path: &str, expected: Error) {
      assert_eq!(path.parse::<RelativePath>().unwrap_err(), expected);
    }

    case("C:", Error::WindowsDiskPrefix { letter: 'C' });

    case("", Error::Empty);
    case(
      ".",
      Error::Component {
        component: ".".into(),
      },
    );
    case(
      "..",
      Error::Component {
        component: "..".into(),
      },
    );
    case("/", Error::LeadingSlash);
    case("foo/", Error::TrailingSlash);
    case("foo//bar", Error::DoubleSlash);
    case("\\", Error::Separator { character: '\\' });
  }

  #[test]
  fn portability() {
    assert!("foo"
      .parse::<RelativePath>()
      .unwrap()
      .check_portability()
      .is_none());
  }

  #[test]
  fn portability_errors() {
    #[track_caller]
    fn case(path: &str, expected: Lint) {
      assert_eq!(
        path
          .parse::<RelativePath>()
          .unwrap()
          .check_portability()
          .unwrap(),
        expected,
      );
    }

    for i in 0..32 {
      let character = char::from_u32(i).unwrap();
      case(
        &character.to_string(),
        Lint::WindowsReservedCharacter { character },
      );
    }

    case("*", Lint::WindowsReservedCharacter { character: '*' });
    case(":", Lint::WindowsReservedCharacter { character: ':' });
    case("<", Lint::WindowsReservedCharacter { character: '<' });
    case(">", Lint::WindowsReservedCharacter { character: '>' });
    case("?", Lint::WindowsReservedCharacter { character: '?' });
    case("\"", Lint::WindowsReservedCharacter { character: '"' });
    case("|", Lint::WindowsReservedCharacter { character: '|' });

    case(&"a".repeat(256), Lint::FilenameLength);

    case("foo/ bar", Lint::WindowsLeadingSpace);

    case("CON", Lint::WindowsReservedFilename { name: "CON".into() });
    case("con", Lint::WindowsReservedFilename { name: "con".into() });
    case(
      "CON./foo",
      Lint::WindowsReservedFilename {
        name: "CON.".into(),
      },
    );
    case(
      "CON.txt",
      Lint::WindowsReservedFilename {
        name: "CON.txt".into(),
      },
    );
    case(
      "CON.txt/foo",
      Lint::WindowsReservedFilename {
        name: "CON.txt".into(),
      },
    );
    case(
      "foo/CON.",
      Lint::WindowsReservedFilename {
        name: "CON.".into(),
      },
    );
    case(
      "foo/CON.txt",
      Lint::WindowsReservedFilename {
        name: "CON.txt".into(),
      },
    );

    case("foo./bar", Lint::WindowsTrailingPeriod);

    case("foo /bar", Lint::WindowsTrailingSpace);
  }

  #[test]
  fn try_from_utf8_path() {
    assert_eq!(
      RelativePath::try_from(Utf8Path::new("..")).unwrap_err(),
      Error::Component {
        component: "..".into()
      }
    );
    assert_eq!(
      RelativePath::try_from(Utf8Path::new("foo/bar")).unwrap(),
      "foo/bar",
    );
  }
}
