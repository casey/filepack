use super::*;

#[repr(transparent)]
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Component(str);

impl Component {
  pub(crate) fn as_path(&self) -> RelativePath {
    self.as_str().parse().unwrap()
  }

  pub(crate) fn as_str(&self) -> &str {
    &self.0
  }

  pub(crate) fn extension(&self) -> Option<&str> {
    match self.0.rfind('.') {
      None | Some(0) => None,
      Some(n) => Some(&self.0[n + 1..]),
    }
  }

  pub(crate) fn new(s: &str) -> Result<&Component, ComponentError> {
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

    Ok(unsafe { &*(std::ptr::from_ref::<str>(s) as *const Component) })
  }
}

impl Borrow<str> for Component {
  fn borrow(&self) -> &str {
    &self.0
  }
}

impl Display for Component {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", &self.0)
  }
}

impl Encode for Component {
  fn encode(&self, encoder: &mut Encoder) {
    self.as_str().encode(encoder);
  }
}

impl PartialEq<&str> for Component {
  fn eq(&self, s: &&str) -> bool {
    self.as_str().eq(*s)
  }
}

impl ToOwned for Component {
  type Owned = ComponentBuf;

  fn to_owned(&self) -> ComponentBuf {
    ComponentBuf::from_component(self)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn current() {
    assert_eq!(
      Component::new(".").unwrap_err(),
      ComponentError::Normal { component: "." },
    );
  }

  #[test]
  fn drive_prefix() {
    assert_eq!(
      Component::new("C:").unwrap_err(),
      ComponentError::WindowsDriveLetter { letter: 'C' },
    );
  }

  #[test]
  fn empty() {
    assert_eq!(Component::new("").unwrap_err(), ComponentError::Empty);
  }

  #[test]
  fn extension() {
    #[track_caller]
    fn case(input: &str, expected: Option<&str>) {
      let component = Component::new(input).unwrap();
      assert_eq!(component.extension(), expected);
    }

    case(".hidden", None);
    case(".hidden.txt", Some("txt"));
    case("file", None);
    case("file.tar.gz", Some("gz"));
    case("file.txt", Some("txt"));
  }

  #[test]
  fn length() {
    let long = "a".repeat(255);
    Component::new(&long).unwrap();

    assert_eq!(
      Component::new(&"a".repeat(256)).unwrap_err(),
      ComponentError::Length,
    );
  }

  #[test]
  fn nul() {
    assert_eq!(Component::new("foo\0bar").unwrap_err(), ComponentError::Nul);
  }

  #[test]
  fn parent() {
    assert_eq!(
      Component::new("..").unwrap_err(),
      ComponentError::Normal { component: ".." },
    );
  }

  #[test]
  fn separator() {
    assert_eq!(
      Component::new("/").unwrap_err(),
      ComponentError::Separator { character: '/' },
    );

    assert_eq!(
      Component::new("\\").unwrap_err(),
      ComponentError::Separator { character: '\\' },
    );
  }
}
