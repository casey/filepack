use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Debug, Default, PartialEq)]
pub(crate) struct Directory {
  pub(crate) version: Version,
  pub(crate) entries: BTreeMap<ComponentBuf, Entry>,
}

impl Decode for Directory {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    let mut decoder = decoder.map::<u8>()?;

    let version = decoder.required_key(0)?;
    let entries = decoder.required_key(1)?;

    decoder.finish()?;

    Ok(Self { version, entries })
  }
}

impl Encode for Directory {
  fn encode(&self, encoder: &mut Encoder) {
    let mut encoder = encoder.map::<u8>(2);
    encoder.item(0, self.version);
    encoder.item(1, &self.entries);
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
        Entry {
          ty: EntryType::File,
          size: 0,
          hash: Hash::bytes(b"bar"),
        },
      )]),
    });
  }
}
