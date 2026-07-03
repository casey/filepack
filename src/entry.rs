use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Debug, Encode, PartialEq)]
pub struct Entry {
  #[n(0)]
  pub ty: EntryType,
  #[n(1)]
  pub hash: Hash,
  #[n(2)]
  pub size: u64,
  #[n(3)]
  pub totals: Option<Totals>,
}

impl Entry {
  pub fn formatted_size(&self) -> SizeFormatter<u64, FormatSizeOptions> {
    let size = if let Some(totals) = &self.totals {
      self
        .size
        .saturating_add(totals.file_size)
        .saturating_add(totals.directory_size)
    } else {
      self.size
    };

    SizeFormatter::new(size, FormatSizeOptions::from(BINARY).decimal_places(1))
  }
}

impl Decode for Entry {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    let mut map = decoder.map::<u64>()?;

    let ty = map.required_key::<EntryType>(0)?;
    let hash = map.required_key(1)?;
    let size = map.required_key(2)?;
    let totals = map.optional_key::<Totals>(3)?;

    map.finish()?;

    ensure!(
      (ty == EntryType::Directory) == totals.is_some(),
      decode_error::EntryTotals { ty },
    );

    Ok(Self {
      ty,
      hash,
      size,
      totals,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_encoding(Entry {
      ty: EntryType::File,
      size: 100,
      hash: Hash::bytes(b"foo"),
      totals: None,
    });

    assert_encoding(Entry {
      ty: EntryType::Directory,
      size: 100,
      hash: Hash::bytes(b"foo"),
      totals: Some(Totals {
        directories: 1,
        directory_size: 100,
        file_size: 3,
        files: 2,
      }),
    });
  }

  #[test]
  fn totals_invariant() {
    #[track_caller]
    fn case(ty: EntryType, totals: bool) {
      let mut encoder = Encoder::new();
      let mut map = encoder.map::<u64>(if totals { 4 } else { 3 });
      map.item(0, ty);
      map.item(1, Hash::bytes(b"foo"));
      map.item(2, 100u64);
      if totals {
        map.item(
          3,
          Totals {
            directories: 1,
            directory_size: 100,
            file_size: 0,
            files: 0,
          },
        );
      }
      drop(map);

      assert_matches!(
        Entry::decode_from_slice(&encoder.finish()),
        Err(DecodeError::EntryTotals { ty: t }) if t == ty,
      );
    }

    case(EntryType::File, true);
    case(EntryType::Directory, false);
  }
}
