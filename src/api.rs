use super::*;

pub(crate) mod gc {
  use super::*;

  #[derive(Debug, Encode, Decode, PartialEq)]
  pub(crate) struct Response {
    #[n(0)]
    pub bytes: u64,
    #[n(1)]
    pub directories: SortedSet<Hash>,
    #[n(2)]
    pub files: SortedSet<Hash>,
  }
}

pub(crate) mod missing {
  use super::*;

  #[derive(Debug, Encode, Decode, PartialEq)]
  pub(crate) struct Request {
    #[n(0)]
    pub hashes: SortedSet<Hash>,
  }

  #[derive(Debug, Encode, Decode, PartialEq)]
  pub(crate) struct Response {
    #[n(0)]
    pub hashes: SortedSet<Hash>,
  }
}

pub(crate) mod packages {
  use super::*;

  #[derive(Debug, Encode, Decode, PartialEq)]
  pub(crate) struct Response {
    #[n(0)]
    pub packages: SortedSet<Fingerprint>,
  }
}
