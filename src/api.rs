use super::*;

pub(crate) mod gc {
  use super::*;

  #[derive(Default, Encode, Decode)]
  pub(crate) struct Response {
    #[n(1)]
    pub(crate) bytes: u64,
    #[n(2)]
    pub(crate) directories: SortedSet<Hash>,
    #[n(3)]
    pub(crate) files: SortedSet<Hash>,
    #[n(4)]
    pub(crate) packages: SortedSet<Fingerprint>,
    #[n(5)]
    pub(crate) revisions: SortedSet<Revision>,
  }
}

pub(crate) mod missing {
  use super::*;

  #[derive(Debug, Encode, Decode)]
  pub(crate) struct Request {
    #[n(1)]
    pub(crate) hashes: SortedSet<Hash>,
  }

  #[derive(Encode, Decode)]
  pub(crate) struct Response {
    #[n(1)]
    pub(crate) hashes: SortedSet<Hash>,
  }
}

pub(crate) mod number {
  use super::*;

  #[derive(Encode, Decode)]
  pub(crate) struct Response {
    #[n(1)]
    pub(crate) package: Fingerprint,
    #[n(2)]
    pub(crate) revision: Revision,
  }
}

pub(crate) mod numbers {
  use super::*;

  #[derive(Default, Encode, Decode)]
  pub(crate) struct Response {
    #[n(1)]
    pub(crate) numbers: SortedSet<u64>,
  }
}

pub(crate) mod packages {
  use super::*;

  #[derive(Default, Encode, Decode)]
  pub(crate) struct Response {
    #[n(1)]
    pub(crate) packages: SortedSet<Fingerprint>,
  }
}

pub(crate) mod revision {
  use super::*;

  #[derive(Debug, Default, Encode, Decode)]
  pub(crate) enum Mode {
    #[default]
    #[n(0)]
    New,
    #[n(1)]
    Replace {
      #[n(1)]
      number: u64,
    },
    #[n(2)]
    Update {
      #[n(1)]
      number: u64,
    },
  }

  #[derive(Debug, Encode, Decode)]
  pub(crate) struct Request {
    #[n(1)]
    pub(crate) mode: Mode,
  }

  #[derive(Encode, Decode)]
  pub(crate) struct Response {
    #[n(1)]
    pub(crate) number: u64,
  }
}
