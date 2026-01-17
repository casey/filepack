use super::*;

#[derive(Clone, Debug, DeserializeFromStr, Eq, Ord, PartialEq, PartialOrd, SerializeDisplay)]
pub(crate) struct Component(String);

impl Component {
  pub(crate) fn as_bytes(&self) -> &[u8] {
    self.0.as_bytes()
  }

  pub(crate) fn as_path(&self) -> RelativePath {
    self.as_str().parse().unwrap()
  }

  pub(crate) fn as_str(&self) -> &str {
    &self.0
  }

  pub(crate) fn check(s: &str) -> Result<(), ComponentError> {
    if s.is_empty() {
      return Err(ComponentError::Empty);
    }

    if s.len() > 255 {
      return Err(ComponentError::Length);
    }

    if s == "." {
      return Err(ComponentError::Normal { component: "." });
    }

    if s == ".." {
      return Err(ComponentError::Normal { component: ".." });
    }

    for character in s.chars() {
      if ['/', '\\'].contains(&character) {
        return Err(ComponentError::Separator { character });
      }

      if character == '\0' {
        return Err(ComponentError::Nul);
      }
    }

    let mut chars = s.chars();
    let first = chars.next();
    let second = chars.next();
    if let Some((first, second)) = first.zip(second)
      && second == ':'
    {
      return Err(ComponentError::WindowsDriveLetter { letter: first });
    }

    Ok(())
  }

  pub(crate) fn extension(&self) -> Option<&str> {
    match self.0.find('.') {
      None | Some(0) => None,
      Some(n) => Some(&self.0[n + 1..]),
    }
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl FromStr for Component {
  type Err = ComponentError;

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
      ComponentError::Normal {
        component: ".".into(),
      },
    );
  }

  #[test]
  fn drive_prefix() {
    assert_eq!(
      "C:".parse::<Component>().unwrap_err(),
      ComponentError::WindowsDriveLetter { letter: 'C' },
    );
  }

  #[test]
  fn empty() {
    assert_eq!("".parse::<Component>().unwrap_err(), ComponentError::Empty,);
  }

  #[test]
  fn length() {
    "a".repeat(255).parse::<Component>().unwrap();

    assert_eq!(
      "a".repeat(256).parse::<Component>().unwrap_err(),
      ComponentError::Length,
    );
  }

  #[test]
  fn nul() {
    assert_eq!(
      "foo\0bar".parse::<Component>().unwrap_err(),
      ComponentError::Nul
    );
  }

  #[test]
  fn parent() {
    assert_eq!(
      "..".parse::<Component>().unwrap_err(),
      ComponentError::Normal {
        component: "..".into(),
      },
    );
  }

  #[test]
  fn separator() {
    assert_eq!(
      "/".parse::<Component>().unwrap_err(),
      ComponentError::Separator { character: '/' },
    );

    assert_eq!(
      "\\".parse::<Component>().unwrap_err(),
      ComponentError::Separator { character: '\\' },
    );
  }
}
