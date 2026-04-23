use super::*;

#[derive(Clone, Copy, Debug, FromRepr, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "kebab-case")]
#[repr(u8)]
pub enum MajorType {
  UnsignedInteger = 0,
  SignedInteger = 1,
  Bytes = 2,
  Text = 3,
  Array = 4,
  Map = 5,
  Tag = 6,
  Value = 7,
}

impl MajorType {
  pub(crate) fn from_initial_byte(initial_byte: u8) -> Self {
    Self::from_repr(initial_byte >> 5 & 0b111).unwrap()
  }

  pub(crate) fn head(self, value: u64) -> Head {
    Head {
      major_type: self,
      value,
    }
  }

  fn name(self) -> &'static str {
    self.into()
  }

  pub(crate) fn value(self) -> u8 {
    self as u8
  }
}

impl Display for MajorType {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name())
  }
}
