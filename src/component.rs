use super::*;

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", transparent)]
pub(crate) struct Component(String);

impl Component {
  pub(crate) fn as_bytes(&self) -> &[u8] {
    self.0.as_bytes()
  }

  pub(crate) fn as_str(&self) -> &str {
    &self.0
  }
}

impl Borrow<str> for Component {
  fn borrow(&self) -> &str {
    self.0.as_str()
  }
}

impl FromStr for Component {
  type Err = PathError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    for character in s.chars() {
      if SEPARATORS.contains(&character) {
        return Err(PathError::Separator { character });
      }
    }

    if s == ".." || s == "." {
      return Err(PathError::Component {
        component: s.into(),
      });
    }

    if s.is_empty() {
      return Err(PathError::ComponentEmpty);
    }

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
  fn empty() {
    assert_eq!(
      "".parse::<Component>().unwrap_err(),
      PathError::ComponentEmpty,
    );
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
