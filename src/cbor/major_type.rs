use super::*;

#[derive(Clone, Copy, Debug, FromRepr, PartialEq)]
#[repr(u8)]
pub(crate) enum MajorType {
  Integer = 0,
  Bytes = 2,
  Text = 3,
  Map = 5,
}

impl MajorType {
  pub(crate) fn head(self, value: u64) -> Head {
    Head {
      major_type: self,
      value,
    }
  }

  pub(crate) fn value(self) -> u8 {
    self as u8
  }

  #[cfg(test)]
  pub(crate) fn from_initial_byte(initial_byte: u8) -> Self {
    Self::from_value(initial_byte >> 5 & 0b111)
  }

  #[cfg(test)]
  pub(crate) fn from_value(value: u8) -> Self {
    Self::from_repr(value).unwrap()
  }
}
