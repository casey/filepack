use super::*;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Message {
  pub fingerprint: Fingerprint,
  pub timestamp: Option<u64>,
}

impl Message {
  pub(crate) fn digest(&self) -> Hash {
    Hash::bytes(&self.encode_to_vec())
  }
}

#[cfg(test)]
impl Decode for Message {
  fn decode(decoder: &mut Decoder) -> Result<Self, DecodeError> {
    let mut map = decoder.map::<u8>()?;

    let application = map.key::<String>(0)?.unwrap();
    ensure!(
      application == "filepack",
      cbor::decode_error::UnexpectedValue
    );

    let ty = map.key::<String>(1)?.unwrap();
    ensure!(ty == "message", cbor::decode_error::UnexpectedValue);

    let fingerprint = map.key::<Fingerprint>(2)?.unwrap();

    let timestamp = map.key::<u64>(3)?;

    map.finish()?;

    Ok(Self {
      fingerprint,
      timestamp,
    })
  }
}

impl Encode for Message {
  fn encode(&self, encoder: &mut Encoder) {
    let length = 3 + count_some!(self.timestamp);
    let mut map = encoder.map::<u8>(length);
    map.item(0, "filepack");
    map.item(1, "message");
    map.item(2, self.fingerprint);
    map.item_optional(3, self.timestamp);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding_with_timestamp() {
    assert_encoding(
      Message {
        fingerprint: Fingerprint::from_bytes([0; Fingerprint::LEN]),
        timestamp: Some(1000),
      },
      &[
        0xA4, 0x00, 0x68, 0x66, 0x69, 0x6C, 0x65, 0x70, 0x61, 0x63, 0x6B, 0x01, 0x67, 0x6D, 0x65,
        0x73, 0x73, 0x61, 0x67, 0x65, 0x02, 0x58, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x19, 0x03, 0xE8,
      ],
    );
  }

  #[test]
  fn encoding_without_timestamp() {
    assert_encoding(
      Message {
        fingerprint: Fingerprint::from_bytes([0; Fingerprint::LEN]),
        timestamp: None,
      },
      &[
        0xA3, 0x00, 0x68, 0x66, 0x69, 0x6C, 0x65, 0x70, 0x61, 0x63, 0x6B, 0x01, 0x67, 0x6D, 0x65,
        0x73, 0x73, 0x61, 0x67, 0x65, 0x02, 0x58, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
      ],
    );
  }

  #[test]
  fn wrong_application() {
    let mut encoder = Encoder::new();
    let mut map = encoder.map::<u8>(3);
    map.item(0, "foo");
    map.item(1, "message");
    map.item(2, Fingerprint::from_bytes([0; Fingerprint::LEN]));
    drop(map);
    let bytes = encoder.finish();

    assert_eq!(
      Message::decode(&mut Decoder::new(bytes)),
      Err(DecodeError::UnexpectedValue),
    );
  }

  #[test]
  fn wrong_type() {
    let mut encoder = Encoder::new();
    let mut map = encoder.map::<u8>(3);
    map.item(0, "filepack");
    map.item(1, "foo");
    map.item(2, Fingerprint::from_bytes([0; Fingerprint::LEN]));
    drop(map);
    let bytes = encoder.finish();

    assert_eq!(
      Message::decode(&mut Decoder::new(bytes)),
      Err(DecodeError::UnexpectedValue),
    );
  }
}
