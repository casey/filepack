use super::*;

const MAX_LEN: usize = 255;

#[derive(Debug, Snafu)]
pub enum PackagerError {
  #[snafu(display("packager may not contain `{}`", character.escape_debug()))]
  Character { character: char },
  #[snafu(display("packager may not be empty"))]
  Empty,
  #[snafu(display("packager may not exceed {max} characters, but has {len}"))]
  Length { max: usize, len: usize },
}

fn forbidden(c: char) -> bool {
  c == '\n' || c == '\t' || c.is_control() || c == '\u{FEFF}' || c == '\u{200B}'
}

#[derive(Clone, Debug, DeserializeFromStr, PartialEq)]
pub(crate) struct Packager(String);

impl FromStr for Packager {
  type Err = PackagerError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s.is_empty() {
      return Err(PackagerError::Empty);
    }

    let len = s.chars().count();
    if len > MAX_LEN {
      return Err(PackagerError::Length { max: MAX_LEN, len });
    }

    if let Some(c) = s.chars().find(|&c| forbidden(c)) {
      return Err(PackagerError::Character { character: c });
    }

    Ok(Self(s.into()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn valid_simple() {
    assert!("Casey Rodarmor".parse::<Packager>().is_ok());
  }

  #[test]
  fn valid_max_length() {
    let s = "a".repeat(255);
    assert!(s.parse::<Packager>().is_ok());
  }

  #[test]
  fn valid_zwj_emoji() {
    assert!("👨‍👩‍👧‍👦".parse::<Packager>().is_ok());
  }

  #[test]
  fn invalid_empty() {
    assert!(matches!(
      "".parse::<Packager>(),
      Err(PackagerError::Empty)
    ));
  }

  #[test]
  fn invalid_too_long() {
    let s = "a".repeat(256);
    assert!(matches!(
      s.parse::<Packager>(),
      Err(PackagerError::Length { max: 255, len: 256 })
    ));
  }

  #[test]
  fn invalid_newline() {
    assert!(matches!(
      "foo\nbar".parse::<Packager>(),
      Err(PackagerError::Character { character: '\n' })
    ));
  }

  #[test]
  fn invalid_tab() {
    assert!(matches!(
      "foo\tbar".parse::<Packager>(),
      Err(PackagerError::Character { character: '\t' })
    ));
  }

  #[test]
  fn invalid_carriage_return() {
    assert!(matches!(
      "foo\rbar".parse::<Packager>(),
      Err(PackagerError::Character { character: '\r' })
    ));
  }

  #[test]
  fn invalid_bom() {
    assert!(matches!(
      "foo\u{FEFF}bar".parse::<Packager>(),
      Err(PackagerError::Character { character: '\u{FEFF}' })
    ));
  }

  #[test]
  fn invalid_zero_width_space() {
    assert!(matches!(
      "foo\u{200B}bar".parse::<Packager>(),
      Err(PackagerError::Character { character: '\u{200B}' })
    ));
  }
}
