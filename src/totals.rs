use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Clone, Copy, Debug, Decode, Default, Encode, PartialEq)]
pub struct Totals {
  #[n(0)]
  pub files: u64,
  #[n(1)]
  pub file_size: u64,
  #[n(2)]
  pub directories: u64,
  #[n(3)]
  pub directory_size: u64,
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
      "{} in {} and {} in {}",
      Count::new(self.file_size, "byte"),
      Count::new(self.files, "file"),
      Count::new(self.directory_size, "byte"),
      Count::irregular(self.directories, "directory", "directories"),
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
        directories: 1,
        directory_size: 2,
        file_size: 3,
        files: 4,
      },
      Totals {
        directories: 5,
        directory_size: 6,
        file_size: 7,
        files: 8,
      },
      Some(Totals {
        directories: 6,
        directory_size: 8,
        file_size: 10,
        files: 12,
      }),
    );

    case(
      Totals {
        directories: u64::MAX,
        ..Totals::default()
      },
      Totals {
        directories: 1,
        ..Totals::default()
      },
      None,
    );

    case(
      Totals {
        directory_size: u64::MAX,
        ..Totals::default()
      },
      Totals {
        directory_size: 1,
        ..Totals::default()
      },
      None,
    );

    case(
      Totals {
        file_size: u64::MAX,
        ..Totals::default()
      },
      Totals {
        file_size: 1,
        ..Totals::default()
      },
      None,
    );

    case(
      Totals {
        files: u64::MAX,
        ..Totals::default()
      },
      Totals {
        files: 1,
        ..Totals::default()
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
      Totals::default(),
      "0 bytes in 0 files and 0 bytes in 0 directories",
    );

    case(
      Totals {
        directories: 1,
        directory_size: 1,
        file_size: 1,
        files: 1,
      },
      "1 byte in 1 file and 1 byte in 1 directory",
    );

    case(
      Totals {
        directories: 2,
        directory_size: 3,
        file_size: 3,
        files: 5,
      },
      "3 bytes in 5 files and 3 bytes in 2 directories",
    );
  }

  #[test]
  fn encoding() {
    assert_cbor(
      Totals {
        directories: 1,
        directory_size: 2,
        file_size: 3,
        files: 4,
      },
      "a40004010302010302",
    );
  }

  #[test]
  fn expect() {
    let actual = Totals {
      directories: 1,
      directory_size: 2,
      file_size: 3,
      files: 4,
    };

    let expected = Totals {
      directories: 5,
      directory_size: 6,
      file_size: 7,
      files: 8,
    };

    assert_eq!(actual.expect(actual), Ok(()));

    assert_eq!(
      actual.expect(expected),
      Err(TotalsError::Mismatch { actual, expected }),
    );
  }
}
