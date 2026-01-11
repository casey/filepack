use super::*;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RelativePath(String);

impl RelativePath {
  const JUNK_NAMES: [&'static str; 2] = [".DS_Store", ".localized"];

  const WINDOWS_RESERVED_CHARACTERS: [char; 7] = ['"', '*', ':', '<', '>', '?', '|'];

  const WINDOWS_RESERVED_NAMES: [&'static str; 28] = [
    "AUX", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "COM¹", "COM²",
    "COM³", "CON", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9", "LPT¹",
    "LPT²", "LPT³", "NUL", "PRN",
  ];

  pub(crate) fn components(&self) -> impl Iterator<Item = Component> {
    self
      .0
      .split('/')
      .map(|component| component.parse().unwrap())
  }

  pub(crate) fn lint(&self) -> Option<Lint> {
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

      if Self::JUNK_NAMES.contains(&component) {
        return Some(Lint::Junk);
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

  pub(crate) fn starts_with(&self, prefix: &RelativePath) -> bool {
    Utf8Path::new(self).starts_with(Utf8Path::new(prefix))
  }

  pub(crate) fn to_lowercase(&self) -> Self {
    Self(self.0.to_lowercase())
  }
}

impl AsRef<str> for RelativePath {
  fn as_ref(&self) -> &str {
    self.0.as_ref()
  }
}

impl AsRef<Utf8Path> for RelativePath {
  fn as_ref(&self) -> &Utf8Path {
    self.0.as_ref()
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
  type Err = PathError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.starts_with('/') {
      return Err(PathError::LeadingSlash);
    }

    if s.ends_with('/') {
      return Err(PathError::TrailingSlash);
    }

    if s.contains("//") {
      return Err(PathError::DoubleSlash);
    }

    let mut chars = s.chars();
    let first = chars.next();
    let second = chars.next();
    if let Some((first, second)) = first.zip(second)
      && second == ':'
    {
      return Err(PathError::WindowsDiskPrefix { letter: first });
    }

    let mut path = String::new();

    for (i, component) in s.split('/').enumerate() {
      if component == ".." || component == "." {
        return Err(PathError::Component {
          component: component.into(),
        });
      }

      for character in component.chars() {
        if SEPARATORS.contains(&character) {
          return Err(PathError::Separator { character });
        }
      }

      if i > 0 {
        path.push('/');
      }

      path.push_str(component);
    }

    if path.is_empty() {
      return Err(PathError::Empty);
    }

    Ok(Self(path))
  }
}

impl PartialEq<&str> for RelativePath {
  fn eq(&self, other: &&str) -> bool {
    self.0 == *other
  }
}

impl AsRef<Path> for RelativePath {
  fn as_ref(&self) -> &Path {
    self.0.as_ref()
  }
}

impl TryFrom<&Utf8Path> for RelativePath {
  type Error = PathError;

  fn try_from(path: &Utf8Path) -> Result<Self, Self::Error> {
    let mut s = String::new();

    for (i, component) in path.components().enumerate() {
      let Utf8Component::Normal(component) = component else {
        return Err(PathError::Component {
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

impl TryFrom<&[&Component]> for RelativePath {
  type Error = PathError;

  fn try_from(components: &[&Component]) -> Result<Self, Self::Error> {
    let mut path = String::new();

    for (i, component) in components.iter().enumerate() {
      if i > 0 {
        path.push('/');
      }
      path.push_str(component.as_str());
    }

    path.parse()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_components() {
    assert_eq!(
      RelativePath::try_from([&"foo".parse().unwrap()].as_slice()).unwrap(),
      "foo",
    );

    assert_eq!(
      RelativePath::try_from([&"foo".parse().unwrap(), &"bar".parse().unwrap()].as_slice())
        .unwrap(),
      "foo/bar",
    );
  }

  #[test]
  fn from_components_drive_prefix() {
    assert_eq!(
      RelativePath::try_from([&"C:".parse().unwrap()].as_slice()).unwrap_err(),
      PathError::WindowsDiskPrefix { letter: 'C' },
    );
  }

  #[test]
  fn from_components_empty() {
    assert_eq!(
      RelativePath::try_from([].as_slice()).unwrap_err(),
      PathError::Empty,
    );
  }

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
    fn case(path: &str, expected: PathError) {
      assert_eq!(path.parse::<RelativePath>().unwrap_err(), expected);
    }

    case("C:", PathError::WindowsDiskPrefix { letter: 'C' });

    case("", PathError::Empty);
    case(
      ".",
      PathError::Component {
        component: ".".into(),
      },
    );
    case(
      "..",
      PathError::Component {
        component: "..".into(),
      },
    );
    case("/", PathError::LeadingSlash);
    case("foo/", PathError::TrailingSlash);
    case("foo//bar", PathError::DoubleSlash);
    case("\\", PathError::Separator { character: '\\' });
  }

  #[test]
  fn lint_fail() {
    #[track_caller]
    fn case(path: &str, expected: Lint) {
      assert_eq!(
        path.parse::<RelativePath>().unwrap().lint().unwrap(),
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

    case(".DS_Store", Lint::Junk);
    case(".localized", Lint::Junk);
  }

  #[test]
  fn lint_pass() {
    assert!("foo".parse::<RelativePath>().unwrap().lint().is_none());
  }

  #[test]
  fn try_from_utf8_path() {
    assert_eq!(
      RelativePath::try_from(Utf8Path::new("..")).unwrap_err(),
      PathError::Component {
        component: "..".into()
      }
    );
    assert_eq!(
      RelativePath::try_from(Utf8Path::new("foo/bar")).unwrap(),
      "foo/bar",
    );
  }
}
