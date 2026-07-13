use super::*;

#[derive(Clone, Copy, Debug, Decode, Default, Encode, PartialEq)]
pub struct Totals {
  #[n(0)]
  pub directories: u64,
  #[n(1)]
  pub directory_size: u64,
  #[n(2)]
  pub file_size: u64,
  #[n(3)]
  pub files: u64,
}

impl Totals {
  pub(crate) fn checked_add(self, other: Self) -> Option<Self> {
    Some(Self {
      directories: self.directories.checked_add(other.directories)?,
      directory_size: self.directory_size.checked_add(other.directory_size)?,
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
    write!(
      f,
      "{} in {}",
      Count(self.file_size, "byte"),
      Count(self.files, "file"),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn checked_add() {
    #[track_caller]
    fn case(a: Totals, b: Totals, expected: Option<Totals>) {
      assert_eq!(a.checked_add(b), expected);
    }

    case(
      Totals {
        file_size: 1,
        files: 2,
      },
      Totals {
        file_size: 3,
        files: 4,
      },
      Some(Totals {
        file_size: 4,
        files: 6,
      }),
    );

    case(
      Totals {
        file_size: u64::MAX,
        files: 0,
      },
      Totals {
        file_size: 1,
        files: 0,
      },
      None,
    );

    case(
      Totals {
        file_size: 0,
        files: u64::MAX,
      },
      Totals {
        file_size: 0,
        files: 1,
      },
      None,
    );
  }

  #[test]
  fn display() {
    #[track_caller]
    fn case(totals: Totals, expected: &str) {
      assert_eq!(totals.to_string(), expected);
    }

    case(
      Totals {
        file_size: 0,
        files: 0,
      },
      "0 bytes in 0 files",
    );

    case(
      Totals {
        file_size: 1,
        files: 1,
      },
      "1 byte in 1 file",
    );

    case(
      Totals {
        file_size: 3,
        files: 2,
      },
      "3 bytes in 2 files",
    );
  }

  #[test]
  fn encoding() {
    assert_cbor(
      Totals {
        file_size: 1,
        files: 2,
      },
      "a200010102",
    );
  }

  #[test]
  fn expect() {
    let actual = Totals {
      file_size: 1,
      files: 2,
    };

    let expected = Totals {
      file_size: 3,
      files: 4,
    };

    assert_eq!(actual.expect(actual), Ok(()));

    assert_eq!(
      actual.expect(expected),
      Err(TotalsError::Mismatch { actual, expected }),
    );
  }
}
