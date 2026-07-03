use super::*;

#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq)]
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
  pub fn new(directory: &Directory) -> Option<Self> {
    let mut totals = Self {
      directories: 0,
      directory_size: 0,
      file_size: 0,
      files: 0,
    };

    for entry in directory.entries.values() {
      match entry.ty {
        EntryType::Directory => {
          let child = entry.totals.unwrap();
          totals.directories = totals
            .directories
            .checked_add(1)?
            .checked_add(child.directories)?;
          totals.directory_size = totals
            .directory_size
            .checked_add(entry.size)?
            .checked_add(child.directory_size)?;
          totals.file_size = totals.file_size.checked_add(child.file_size)?;
          totals.files = totals.files.checked_add(child.files)?;
        }
        EntryType::File => {
          totals.file_size = totals.file_size.checked_add(entry.size)?;
          totals.files = totals.files.checked_add(1)?;
        }
      }
    }

    Some(totals)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_cbor(
      Totals {
        directories: 1,
        directory_size: 2,
        file_size: 4,
        files: 3,
      },
      "a40001010202040303",
    );
  }
}
