use super::*;

pub(crate) mod gc {
  use super::*;

  #[derive(Default, Encode, Decode)]
  pub(crate) struct Response {
    /// Bytes freed.
    #[n(1)]
    pub(crate) bytes: u64,
    /// Hashes of the deleted directory objects.
    #[n(2)]
    pub(crate) directories: SortedSet<Hash>,
    /// Hashes of the deleted files.
    #[n(3)]
    pub(crate) files: SortedSet<Hash>,
    /// Fingerprints of the deleted packages.
    #[n(4)]
    pub(crate) packages: SortedSet<Fingerprint>,
    /// Revisions of the deleted packages.
    #[n(5)]
    pub(crate) revisions: SortedSet<Revision>,
  }
}

pub(crate) mod missing {
  use super::*;

  #[derive(Debug, Encode, Decode)]
  pub(crate) struct Request {
    /// Hashes to check for.
    #[n(1)]
    pub(crate) hashes: SortedSet<Hash>,
  }

  #[derive(Encode, Decode)]
  pub(crate) struct Response {
    /// Hashes not present on the server.
    #[n(1)]
    pub(crate) hashes: SortedSet<Hash>,
  }
}

pub(crate) mod number {
  use super::*;

  #[derive(Encode, Decode)]
  pub(crate) struct Response {
    /// Fingerprint of the package.
    #[n(1)]
    pub(crate) package: Fingerprint,
    /// Revision with the given number.
    #[n(2)]
    pub(crate) revision: Revision,
  }
}

pub(crate) mod numbers {
  use super::*;

  #[derive(Default, Encode, Decode)]
  pub(crate) struct Response {
    /// Package numbers present on the server.
    #[n(1)]
    pub(crate) numbers: SortedSet<u64>,
  }
}

pub(crate) mod packages {
  use super::*;

  #[derive(Default, Encode, Decode)]
  pub(crate) struct Response {
    /// Fingerprints of packages on the server.
    #[n(1)]
    pub(crate) packages: SortedSet<Fingerprint>,
  }
}

pub(crate) mod revision {
  use super::*;

  #[derive(Debug, Default, Encode, Decode)]
  pub(crate) enum Mode {
    /// Upload as a new package.
    #[default]
    #[n(0)]
    New,
    /// Replace package `number`.
    #[n(1)]
    Replace {
      /// Number of the package to replace.
      #[n(1)]
      number: u64,
    },
    /// Add a revision to package `number`.
    #[n(2)]
    Update {
      /// Number of the package to update.
      #[n(1)]
      number: u64,
    },
  }

  #[derive(Debug, Encode, Decode)]
  pub(crate) struct Request {
    /// How the uploaded package relates to existing packages.
    #[n(1)]
    pub(crate) mode: Mode,
  }

  #[derive(Encode, Decode)]
  pub(crate) struct Response {
    /// Number assigned to the uploaded package.
    #[n(1)]
    pub(crate) number: u64,
  }
}
