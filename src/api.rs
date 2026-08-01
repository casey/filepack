use super::*;

pub mod missing {
  use super::*;

  #[derive(Debug, Encode, Decode, PartialEq)]
  pub struct Request {
    #[n(0)]
    pub hashes: SortedSet<Hash>,
  }

  #[derive(Debug, Encode, Decode, PartialEq)]
  pub struct Response {
    #[n(0)]
    pub hashes: SortedSet<Hash>,
  }
}

pub mod packages {
  use super::*;

  #[derive(Debug, Encode, Decode, PartialEq)]
  pub struct Response {
    #[n(0)]
    pub packages: SortedSet<Fingerprint>,
  }
}
