use super::*;

pub(crate) use self::error::Error;

mod error;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct RelativePath(String);

impl RelativePath {
  const ILLEGAL_CHARACTERS: [char; 2] = ['/', '\\'];

  const NON_PORTABLE_CHARACTERS: [char; 7] = ['"', '*', ':', '<', '>', '?', '|'];

  const NON_PORTABLE_NAMES: [&'static str; 22] = [
    "aux", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8", "com9", "con", "lpt1",
    "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9", "nul", "prn",
  ];

  pub(crate) fn check_portability(&self) -> Result<(), Lint> {
    for component in Utf8Path::new(&self.0).components() {
      let Utf8Component::Normal(component) = component else {
        unreachable!(
          "it is not possible to construct a `RelativePath` containing non-normal components",
        );
      };

      for character in component.chars() {
        if Self::NON_PORTABLE_CHARACTERS.contains(&character) {
          return Err(Lint::Character { character });
        }

        if let 0..32 = character as u32 {
          return Err(Lint::Character { character });
        }
      }

      for name in Self::NON_PORTABLE_NAMES {
        let lowercase = component.to_lowercase();

        if lowercase == name {
          return Err(Lint::Name {
            name: component.into(),
          });
        }

        if lowercase.starts_with(name) && lowercase.chars().nth(name.chars().count()) == Some('.') {
          return Err(Lint::Name {
            name: component.into(),
          });
        }
      }

      if component.starts_with(' ') {
        return Err(Lint::LeadingSpace);
      }

      if component.ends_with(' ') {
        return Err(Lint::TrailingSpace);
      }

      if component.ends_with('.') {
        return Err(Lint::TrailingPeriod);
      }

      if component.len() > 255 {
        return Err(Lint::ComponentLength {
          length: component.len(),
        });
      }
    }

    Ok(())
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

impl AsRef<Path> for RelativePath {
  fn as_ref(&self) -> &Path {
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
        if Self::ILLEGAL_CHARACTERS.contains(&character) {
          return Err(Error::Character { character });
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
    case("\\", Error::Character { character: '\\' });
  }

  #[test]
  fn portability() {
    "foo"
      .parse::<RelativePath>()
      .unwrap()
      .check_portability()
      .unwrap();
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
          .unwrap_err(),
        expected,
      );
    }

    for i in 0..32 {
      let character = char::from_u32(i).unwrap();
      case(&character.to_string(), Lint::Character { character });
    }

    case("*", Lint::Character { character: '*' });
    case(":", Lint::Character { character: ':' });
    case("<", Lint::Character { character: '<' });
    case(">", Lint::Character { character: '>' });
    case("?", Lint::Character { character: '?' });
    case("\"", Lint::Character { character: '"' });
    case("|", Lint::Character { character: '|' });

    case(&"a".repeat(256), Lint::ComponentLength { length: 256 });

    case("foo/ bar", Lint::LeadingSpace);

    case("CON", Lint::Name { name: "CON".into() });
    case("con", Lint::Name { name: "con".into() });
    case(
      "CON./foo",
      Lint::Name {
        name: "CON.".into(),
      },
    );
    case(
      "CON.txt",
      Lint::Name {
        name: "CON.txt".into(),
      },
    );
    case(
      "CON.txt/foo",
      Lint::Name {
        name: "CON.txt".into(),
      },
    );
    case(
      "foo/CON.",
      Lint::Name {
        name: "CON.".into(),
      },
    );
    case(
      "foo/CON.txt",
      Lint::Name {
        name: "CON.txt".into(),
      },
    );

    case("foo./bar", Lint::TrailingPeriod);

    case("foo /bar", Lint::TrailingSpace);
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
