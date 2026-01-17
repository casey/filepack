use super::*;

#[derive(Clone, Debug, DeserializeFromStr, Eq, Ord, PartialEq, PartialOrd, SerializeDisplay)]
pub(crate) struct Component(String);

impl Component {
  pub(crate) fn as_bytes(&self) -> &[u8] {
    self.0.as_bytes()
  }

  pub(crate) fn as_str(&self) -> &str {
    &self.0
  }

  pub(crate) fn check(s: &str) -> Result<(), PathError> {
    if s.is_empty() {
      return Err(PathError::ComponentEmpty);
    }

    if s.len() > 255 {
      return Err(PathError::Length);
    }

    if s == ".." || s == "." {
      return Err(PathError::Component {
        component: s.into(),
      });
    }

    for character in s.chars() {
      if ['/', '\\'].contains(&character) {
        return Err(PathError::Separator { character });
      }

      if character == '\0' {
        return Err(PathError::Nul);
      }
    }

    let mut chars = s.chars();
    let first = chars.next();
    let second = chars.next();
    if let Some((first, second)) = first.zip(second)
      && second == ':'
    {
      return Err(PathError::WindowsDiskPrefix { letter: first });
    }

    Ok(())
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl FromStr for Component {
  type Err = PathError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::check(s)?;
    Ok(Self(s.into()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn current() {
    assert_eq!(
      ".".parse::<Component>().unwrap_err(),
      PathError::Component {
        component: ".".into(),
      },
    );
  }

  #[test]
  fn drive_prefix() {
    assert_eq!(
      "C:".parse::<Component>().unwrap_err(),
      PathError::WindowsDiskPrefix { letter: 'C' },
    );
  }

  #[test]
  fn empty() {
    assert_eq!(
      "".parse::<Component>().unwrap_err(),
      PathError::ComponentEmpty,
    );
  }

  #[test]
  fn length() {
    "a".repeat(255).parse::<Component>().unwrap();

    assert_eq!(
      "a".repeat(256).parse::<Component>().unwrap_err(),
      PathError::Length,
    );
  }

  #[test]
  fn nul() {
    assert_eq!("foo\0bar".parse::<Component>().unwrap_err(), PathError::Nul);
  }

  #[test]
  fn parent() {
    assert_eq!(
      "..".parse::<Component>().unwrap_err(),
      PathError::Component {
        component: "..".into(),
      },
    );
  }

  #[test]
  fn separator() {
    assert_eq!(
      "/".parse::<Component>().unwrap_err(),
      PathError::Separator { character: '/' },
    );

    assert_eq!(
      "\\".parse::<Component>().unwrap_err(),
      PathError::Separator { character: '\\' },
    );
  }
}
