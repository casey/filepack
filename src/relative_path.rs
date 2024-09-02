use super::*;

// todo:
// - distinguish between illegal and non portable errors
// - error on `X:` prefix

#[derive(Debug, PartialEq, Snafu)]
pub(crate) enum Error {
  #[snafu(display("leading slash"))]
  LeadingSlash,
  #[snafu(display("trailing slash"))]
  TrailingSlash,
  #[snafu(display("Windows disk prefix `{letter}:`"))]
  DiskPrefix {
    letter: char,
  },
  #[snafu(display("double slash"))]
  DoubleSlash,
  #[snafu(display("illegal character `{character}`"))]
  Character {
    character: char,
  },
  #[snafu(display("illegal path component `{}`", component))]
  Component {
    component: String,
  },
  #[snafu(display("length {length} path component"))]
  ComponentLength {
    length: usize,
  },
  #[snafu(display("empty path"))]
  Empty,
  #[snafu(display("leading space"))]
  LeadingSpace,
  Name {
    name: String,
  },
  TrailingPeriod,
  TrailingSpace,
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct RelativePath(String);

impl RelativePath {
  const ILLEGAL_CHARACTERS: [char; 2] = ['/', '\\'];

  const NON_PORTABLE_CHARACTERS: [char; 7] = ['"', '*', ':', '<', '>', '?', '|'];

  const NON_PORTABLE_NAMES: [&'static str; 22] = [
    "aux", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8", "com9", "con", "lpt1",
    "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9", "nul", "prn",
  ];

  fn check_portability(&self) -> Result<(), Error> {
    for component in Utf8Path::new(&self.0).components() {
      let Utf8Component::Normal(component) = component else {
        unreachable!(
          "it is not possible to construct a `RelativePath` containing non-normal components",
        );
      };

      for character in component.chars() {
        if Self::NON_PORTABLE_CHARACTERS.contains(&character) {
          return Err(Error::Character { character });
        }

        if let 0..32 = character as u32 {
          return Err(Error::Character { character });
        }
      }

      for name in Self::NON_PORTABLE_NAMES {
        let lowercase = component.to_lowercase();

        if lowercase == name {
          return Err(Error::Name {
            name: component.into(),
          });
        }

        if lowercase.starts_with(name) {
          if lowercase.chars().nth(name.chars().count()) == Some('.') {
            return Err(Error::Name {
              name: component.into(),
            });
          }
        }
      }

      if component.starts_with(' ') {
        return Err(Error::LeadingSpace);
      }

      if component.ends_with(' ') {
        return Err(Error::TrailingSpace);
      }

      if component.ends_with('.') {
        return Err(Error::TrailingPeriod);
      }

      if component.len() > 255 {
        return Err(Error::ComponentLength {
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

impl<'de> Deserialize<'de> for RelativePath {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    use serde::de::{Error, Unexpected};

    let s = String::deserialize(deserializer)?;

    Ok(
      s.parse::<Self>()
        .map_err(|err| D::Error::invalid_value(Unexpected::Str(&s), &err.to_string().as_str()))?,
    )
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
        return Err(Error::DiskPrefix { letter: first });
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

    case("C:", Error::DiskPrefix { letter: 'C' });

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
    fn case(path: &str, expected: Error) {
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
      case(&character.to_string(), Error::Character { character });
    }

    case("*", Error::Character { character: '*' });
    case(":", Error::Character { character: ':' });
    case("<", Error::Character { character: '<' });
    case(">", Error::Character { character: '>' });
    case("?", Error::Character { character: '?' });
    case("\"", Error::Character { character: '"' });
    case("|", Error::Character { character: '|' });

    case(&"a".repeat(256), Error::ComponentLength { length: 256 });

    case("foo/ bar", Error::LeadingSpace);

    case("CON", Error::Name { name: "CON".into() });
    case("con", Error::Name { name: "con".into() });
    case(
      "CON./foo",
      Error::Name {
        name: "CON.".into(),
      },
    );
    case(
      "CON.txt",
      Error::Name {
        name: "CON.txt".into(),
      },
    );
    case(
      "CON.txt/foo",
      Error::Name {
        name: "CON.txt".into(),
      },
    );
    case(
      "foo/CON.",
      Error::Name {
        name: "CON.".into(),
      },
    );
    case(
      "foo/CON.txt",
      Error::Name {
        name: "CON.txt".into(),
      },
    );

    case("foo./bar", Error::TrailingPeriod);

    case("foo /bar", Error::TrailingSpace);
  }
}
