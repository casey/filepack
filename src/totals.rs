use super::*;

#[derive(Clone, Copy, Debug, Decode, Default, Encode, PartialEq)]
pub struct Totals {
  #[n(0)]
  pub(crate) file_size: u64,
  #[n(1)]
  pub(crate) files: u64,
}

impl Totals {
  pub(crate) fn checked_add(self, other: Self) -> Option<Self> {
    Some(Self {
      file_size: self.file_size.checked_add(other.file_size)?,
      files: self.files.checked_add(other.files)?,
    })
  }

  pub(crate) fn expect(self, expected: Totals) -> Result<(), TotalsError> {
    ensure! {
      self == expected,
      totals_error::Mismatch { actual: self, expected },
    }

    Ok(())
  }
}

impl Display for Totals {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{} in {} files", format_size(self.file_size), self.files)
  }
}
