use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Clone, Debug, Default, Encode, Decode, PartialEq)]
pub struct Directory {
  #[n(0)]
  pub(crate) version: Version,
  #[n(1)]
  pub(crate) entries: BTreeMap<ComponentBuf, Entry>,
}

impl Directory {
  pub fn new() -> Self {
    Self::default()
  }

  pub(crate) fn totals(&self) -> Result<Totals, TotalsError> {
    let mut totals = Totals::default();

    for entry in self.entries.values() {
      let entry_totals = match entry {
        Entry::File { size, .. } => Totals {
          directories: 0,
          directory_size: 0,
          file_size: *size,
          files: 1,
        },
        Entry::Directory { size, totals, .. } => totals
          .checked_add(Totals {
            directories: 1,
            directory_size: *size,
            file_size: 0,
            files: 0,
          })
          .context(totals_error::Overflow)?,
      };

      totals = totals
        .checked_add(entry_totals)
        .context(totals_error::Overflow)?;
    }

    Ok(totals)
  }

  pub(crate) fn with_entries(entries: BTreeMap<ComponentBuf, Entry>) -> Self {
    Self {
      version: Version::Zero,
      entries,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_encoding(Directory {
      version: Version::Zero,
      entries: BTreeMap::from([(
        "foo".parse::<ComponentBuf>().unwrap(),
        Entry::file(Hash::bytes(b"bar"), 0),
      )]),
    });
  }

  #[test]
  fn totals() {
    assert_eq!(Directory::new().totals(), Ok(Totals::default()));

    let mut subdirectory = Directory::new();
    subdirectory.insert_file("foo", b"xy");

    let (cbor, _hash) = subdirectory.cbor();

    let mut directory = Directory::new();
    directory
      .insert_file("bar", b"x")
      .insert_directory("baz", &subdirectory);

    assert_eq!(
      directory.totals(),
      Ok(Totals {
        directories: 1,
        directory_size: cbor.len().into_u64(),
        file_size: 3,
        files: 2,
      }),
    );

    let hash = Hash::bytes(b"foo");

    let mut directory = Directory::new();
    directory
      .insert_entry("bar", Entry::file(hash, u64::MAX))
      .insert_entry(
        "baz",
        Entry::Directory {
          hash,
          size: 0,
          totals: Totals {
            directories: 0,
            directory_size: 0,
            file_size: 1,
            files: 1,
          },
        },
      );

    assert_eq!(directory.totals(), Err(TotalsError::Overflow));

    let mut directory = Directory::new();
    directory
      .insert_entry(
        "bar",
        Entry::Directory {
          hash,
          size: 0,
          totals: Totals {
            directories: 0,
            directory_size: 0,
            file_size: 0,
            files: u64::MAX,
          },
        },
      )
      .insert_entry("baz", Entry::file(hash, 0));

    assert_eq!(directory.totals(), Err(TotalsError::Overflow));

    let mut directory = Directory::new();
    directory.insert_entry(
      "bar",
      Entry::Directory {
        hash,
        size: 1,
        totals: Totals {
          directories: 0,
          directory_size: u64::MAX,
          file_size: 0,
          files: 0,
        },
      },
    );

    assert_eq!(directory.totals(), Err(TotalsError::Overflow));

    let mut directory = Directory::new();
    directory.insert_entry(
      "bar",
      Entry::Directory {
        hash,
        size: 0,
        totals: Totals {
          directories: u64::MAX,
          directory_size: 0,
          file_size: 0,
          files: 0,
        },
      },
    );

    assert_eq!(directory.totals(), Err(TotalsError::Overflow));
  }
}
