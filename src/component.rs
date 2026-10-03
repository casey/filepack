use super::*;

#[repr(transparent)]
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Component(str);

impl Component {
  const RESERVED: &'static [&'static str] = &[Manifest::FILENAME, State::DIR];

  pub(crate) fn as_str(&self) -> &str {
    &self.0
  }

  fn cast(s: &str) -> &Component {
    unsafe { &*(ptr::from_ref::<str>(s) as *const Component) }
  }

  pub(crate) fn extension(&self) -> Option<&str> {
    match self.0.rfind('.') {
      None | Some(0) => None,
      Some(n) => Some(&self.0[n + 1..]),
    }
  }

  pub(crate) fn from_component_buf(c: &ComponentBuf) -> &Self {
    Self::cast(c.borrow())
  }

  pub(crate) fn is_reserved(&self) -> bool {
    Self::RESERVED
      .iter()
      .any(|name| self.0.eq_ignore_ascii_case(name))
  }

  pub(crate) fn new(s: &str) -> Result<&Component, ComponentError> {
    if s.is_empty() {
      return Err(ComponentError::Empty);
    }

    if s == "." {
      return Err(ComponentError::Current);
    }

    if s == ".." {
      return Err(ComponentError::Parent);
    }

    for character in s.chars() {
      if ['/', '\\'].contains(&character) {
        return Err(ComponentError::Separator { character });
      }

      if character.is_control() {
        return Err(ComponentError::Control { character });
      }
    }

    let mut chars = s.chars();
    let first = chars.next();
    let second = chars.next();
    if let Some((first, second)) = first.zip(second)
      && first.is_ascii_alphabetic()
      && second == ':'
    {
      return Err(ComponentError::WindowsDriveLetter { letter: first });
    }

    Ok(Self::cast(s))
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
  fn control() {
    #[track_caller]
    fn case(character: char) {
      assert_eq!(
        Component::new(&format!("foo{character}bar")).unwrap_err(),
        ComponentError::Control { character },
      );
    }

    case('\u{00}');
    case('\u{1f}');
    case('\u{7f}');
    case('\u{9f}');
  }

  #[test]
  fn current() {
    assert_eq!(Component::new(".").unwrap_err(), ComponentError::Current);
  }

  #[test]
  fn drive_prefix() {
    assert_eq!(
      Component::new("C:").unwrap_err(),
      ComponentError::WindowsDriveLetter { letter: 'C' },
    );

    assert_eq!(
      Component::new("c:foo").unwrap_err(),
      ComponentError::WindowsDriveLetter { letter: 'c' },
    );

    Component::new("3:45").unwrap();
    Component::new("é:").unwrap();
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
  fn is_reserved() {
    #[track_caller]
    fn case(input: &str, expected: bool) {
      assert_eq!(Component::new(input).unwrap().is_reserved(), expected);
    }

    case("manifest.filepack", true);
    case(".filepack", true);
    case("MANIFEST.FILEPACK", true);
    case("foo", false);
  }

  #[test]
  fn parent() {
    assert_eq!(Component::new("..").unwrap_err(), ComponentError::Parent);
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
