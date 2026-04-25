use super::*;

#[derive(Clone, Debug, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub struct Message {
  #[n(0)]
  pub fingerprint: Fingerprint,
  #[n(1)]
  pub timestamp: Option<u64>,
}

impl Message {
  pub(crate) fn digest(&self) -> Hash {
    let envelope = Envelope {
      application: "filepack",
      ty: "message",
      message: self.clone(),
    };

    Hash::bytes(&envelope.encode_to_vec())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding_with_timestamp() {
    assert_encoding(Message {
      fingerprint: Fingerprint::from_bytes([0; Fingerprint::LEN]),
      timestamp: Some(1000),
    });
  }

  #[test]
  fn encoding_without_timestamp() {
    assert_encoding(Message {
      fingerprint: Fingerprint::from_bytes([0; Fingerprint::LEN]),
      timestamp: None,
    });
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

    assert_matches!(
      Message::decode(&mut Decoder::new(&bytes)),
      Err(DecodeError::UnexpectedValue { actual, expected: "filepack" }) if actual == "foo",
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

    assert_matches!(
      Message::decode(&mut Decoder::new(&bytes)),
      Err(DecodeError::UnexpectedValue { actual, expected: "message" }) if actual == "foo",
    );
  }
}
