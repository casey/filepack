use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fourcc(pub(crate) [u8; 4]);

impl Fourcc {
  pub(crate) fn new(name: &str) -> Self {
    Self(
      name
        .chars()
        .map(|character| u8::try_from(character).expect("fourcc name must be Latin-1"))
        .collect::<Vec<u8>>()
        .try_into()
        .expect("fourcc name must be four characters"),
    )
  }
}

impl Display for Fourcc {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    for byte in self.0 {
      if byte.is_ascii_control() || byte == 0x7f {
        write!(f, "\\x{byte:02x}")?;
      } else {
        write!(f, "{}", char::from(byte))?;
      }
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(Fourcc(*b"mvhd").to_string(), "mvhd");
    assert_eq!(Fourcc(*b"\xa9nam").to_string(), "©nam");
    assert_eq!(Fourcc(*b"\0abc").to_string(), "\\x00abc");
    assert_eq!(Fourcc(*b"a\x7fbc").to_string(), "a\\x7fbc");
  }

  #[test]
  fn new() {
    assert_eq!(Fourcc::new("foob"), Fourcc(*b"foob"));
    assert_eq!(Fourcc::new("©nam"), Fourcc(*b"\xa9nam"));
  }

  #[test]
  #[should_panic(expected = "fourcc name must be four characters")]
  fn new_length() {
    Fourcc::new("foo");
  }

  #[test]
  #[should_panic(expected = "fourcc name must be Latin-1")]
  fn new_non_latin_1() {
    Fourcc::new("€foo");
  }
}
