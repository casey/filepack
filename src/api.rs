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

pub(crate) mod number {
  use super::*;

  #[derive(Debug, Encode, Decode, PartialEq)]
  pub(crate) struct Response {
    #[n(0)]
    pub(crate) package: Fingerprint,
    #[n(1)]
    pub(crate) revision: Revision,
  }
}

pub(crate) mod numbers {
  use super::*;

  #[derive(Debug, Default, Encode, Decode, PartialEq)]
  pub(crate) struct Response {
    #[n(0)]
    pub(crate) numbers: SortedSet<u64>,
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

pub(crate) mod revision {
  use super::*;

  #[derive(Debug, Default, Encode, Decode, PartialEq)]
  pub(crate) enum Mode {
    #[default]
    #[n(0)]
    New,
    #[n(1)]
    Replace {
      #[n(0)]
      number: u64,
    },
    #[n(2)]
    Update {
      #[n(0)]
      number: u64,
    },
  }

  #[derive(Debug, Default, Encode, Decode, PartialEq)]
  pub(crate) struct Request {
    #[n(0)]
    pub(crate) mode: Mode,
  }

  #[derive(Debug, Encode, Decode, PartialEq)]
  pub(crate) struct Response {
    #[n(0)]
    pub(crate) number: u64,
  }
}
