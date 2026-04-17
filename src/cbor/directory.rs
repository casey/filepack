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
    assert_encoding(
      Directory {
        version: Version::Zero,
        entries: BTreeMap::from([(
          "foo".parse::<Component>().unwrap(),
          Entry {
            ty: EntryType::File,
            size: 0,
            hash: Hash::bytes(b"bar"),
          },
        )]),
      },
      &[
        0xA2, 0x00, 0x00, 0x01, 0xA1, 0x63, 0x66, 0x6F, 0x6F, 0xA3, 0x00, 0x00, 0x01, 0x58, 0x20,
        0xF2, 0xE8, 0x97, 0xEE, 0xD7, 0xD2, 0x06, 0xCD, 0x85, 0x5D, 0x44, 0x15, 0x98, 0xFA, 0x52,
        0x1A, 0xBC, 0x75, 0xAA, 0x96, 0x95, 0x3E, 0x97, 0xC0, 0x30, 0xC9, 0x61, 0x2C, 0x30, 0xC1,
        0x29, 0x3D, 0x02, 0x00,
      ],
    );
  }
}
