use super::*;

pub(crate) mod gc {
  use super::*;

  #[derive(Debug, Default, Encode, Decode, PartialEq)]
  pub(crate) struct Response {
    #[n(0)]
    pub(crate) bytes: u64,
    #[n(1)]
    pub(crate) directories: SortedSet<Hash>,
    #[n(2)]
    pub(crate) files: SortedSet<Hash>,
    #[n(3)]
    pub(crate) revisions: SortedSet<Revision>,
  }
}

pub(crate) mod missing {
  use super::*;

  #[derive(Debug, Encode, Decode, PartialEq)]
  pub(crate) struct Request {
    #[n(0)]
    pub(crate) hashes: SortedSet<Hash>,
  }

  #[derive(Debug, Encode, Decode, PartialEq)]
  pub(crate) struct Response {
    #[n(0)]
    pub(crate) hashes: SortedSet<Hash>,
  }
}

pub(crate) mod package {
  use super::*;

  #[derive(Debug, Default, Encode, Decode, PartialEq)]
  pub(crate) struct Request {
    #[n(0)]
    pub(crate) replace: Option<u64>,
  }

  #[derive(Debug, Encode, Decode, PartialEq)]
  pub(crate) struct Response {
    #[n(0)]
    pub(crate) number: u64,
  }
}

pub(crate) mod packages {
  use super::*;

  #[derive(Debug, Default, Encode, Decode, PartialEq)]
  pub(crate) struct Response {
    #[n(0)]
    pub(crate) packages: SortedSet<Fingerprint>,
  }
}
