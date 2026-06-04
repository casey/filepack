use super::*;

pub mod missing {
  use super::*;

  #[derive(Debug, Encode, Decode, PartialEq)]
  pub struct Request {
    #[n(0)]
    pub hashes: Unique<Hash>,
  }

  #[derive(Debug, Encode, Decode, PartialEq)]
  pub struct Response {
    #[n(0)]
    pub hashes: Unique<Hash>,
  }

  #[cfg(test)]
  mod tests {
    use super::*;

    #[test]
    fn request_encoding() {
      assert_encoding(Request {
        hashes: BTreeSet::from([Hash::bytes(b"foo"), Hash::bytes(b"bar")]).into(),
      });
    }

    #[test]
    fn response_encoding() {
      assert_encoding(Response {
        hashes: BTreeSet::from([Hash::bytes(b"foo"), Hash::bytes(b"bar")]).into(),
      });
    }
  }
}
