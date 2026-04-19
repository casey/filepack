use super::*;

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Debug, PartialEq)]
pub(crate) struct Directory {
  pub(crate) version: Version,
  pub(crate) entries: BTreeMap<Component, Entry>,
}

impl Directory {
  pub(crate) fn hash(directory: &crate::Directory) -> (Hash, u64) {
    let entries = directory
      .entries
      .iter()
      .map(|(component, entry)| (component.clone(), Entry::new(entry)))
      .collect::<BTreeMap<Component, Entry>>();

    let size = entries.values().map(|entry| entry.size).sum();

    let cbor = Self {
      version: Version::Zero,
      entries,
    }
    .encode_to_vec();

    let hash = Hash::bytes(&cbor);

    (hash, size)
  }
}

#[cfg(test)]
impl Decode for Directory {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    let mut decoder = decoder.map::<u8>()?;

    let version = decoder.key(0)?.unwrap();
    let entries = decoder.key(1)?.unwrap();

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
        "foo".parse::<Component>().unwrap(),
        Entry {
          ty: EntryType::File,
          size: 0,
          hash: Hash::bytes(b"bar"),
        },
      )]),
    });
  }
}
