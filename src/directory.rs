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
          file_size: *size,
          files: 1,
        },
        Entry::Directory { totals, .. } => *totals,
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
  fn total_file_size() {
    assert_eq!(Directory::new().total_file_size(), Some(0));

    let mut subdirectory = Directory::new();
    subdirectory.insert_file("foo", b"xy");

    let mut directory = Directory::new();
    directory
      .insert_file("bar", b"x")
      .insert_directory("baz", &subdirectory);

    assert_eq!(directory.total_file_size(), Some(3));

    let hash = Hash::bytes(b"foo");

    let mut directory = Directory::new();
    directory
      .insert_entry("bar", Entry::file(hash, u64::MAX))
      .insert_entry(
        "baz",
        Entry::Directory {
          hash,
          size: 0,
          total_file_size: 1,
        },
      );

    assert_eq!(directory.total_file_size(), None);
  }
}
