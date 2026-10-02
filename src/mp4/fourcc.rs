use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Fourcc(pub(crate) [u8; 4]);

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
}
