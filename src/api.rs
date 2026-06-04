use super::*;

pub mod missing {
  use super::*;

  #[derive(Debug, Encode, Decode, PartialEq)]
  pub struct Request {
    #[n(0)]
    pub hashes: Vec<Hash>,
  }

  #[derive(Debug, Encode, Decode, PartialEq)]
  pub struct Response {
    #[n(0)]
    pub hashes: Vec<Hash>,
  }

  #[cfg(test)]
  mod tests {
    use super::*;

    #[test]
    fn request_encoding() {
      assert_encoding(Request {
        hashes: vec![Hash::bytes(b"foo"), Hash::bytes(b"bar")],
      });
    }

    #[test]
    fn response_encoding() {
      assert_encoding(Response {
        hashes: vec![Hash::bytes(b"foo"), Hash::bytes(b"bar")],
      });
    }
  }
}
