use super::*;

#[derive(Clone, Debug, Decode, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub struct Statement {
  #[n(0)]
  pub fingerprint: Fingerprint,
  #[n(1)]
  pub timestamp: Option<u64>,
}

impl Statement {
  pub(crate) fn digest(&self) -> Hash {
    let envelope = Envelope {
      application: Application::Filepack,
      context: Context::Statement,
      statement: self.clone(),
    };

    Hash::bytes(&envelope.encode_to_vec())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[track_caller]
  fn case(message: Statement) {
    let mut encoder = Encoder::new();

    {
      let mut encoder = encoder.map::<u64>(3);
      encoder.item(0, "filepack");
      encoder.item(1, "message");
      encoder.item(2, &message);
    }

    let bytes = encoder.finish();

    assert_eq!(message.digest(), Hash::bytes(&bytes));
  }

  #[test]
  fn digest_with_timestamp() {
    case(Statement {
      fingerprint: Fingerprint::from_bytes([0; Fingerprint::LEN]),
      timestamp: Some(1000),
    });
  }

  #[test]
  fn digest_without_timestamp() {
    case(Statement {
      fingerprint: Fingerprint::from_bytes([0; Fingerprint::LEN]),
      timestamp: None,
    });
  }
}
