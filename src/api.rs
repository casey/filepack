use super::*;

pub(crate) mod gc {
  use super::*;

  /// Response body of `POST /api/gc`.
  #[derive(Default, Encode, Decode)]
  pub(crate) struct Response {
    /// Bytes freed.
    #[n(1)]
    pub(crate) bytes: u64,
    /// Hashes of the deleted directories.
    #[n(2)]
    pub(crate) directories: SortedSet<Hash>,
    /// Hashes of the deleted files.
    #[n(3)]
    pub(crate) files: SortedSet<Hash>,
    /// Fingerprints of the deleted packages.
    #[n(4)]
    pub(crate) packages: SortedSet<Fingerprint>,
    /// Hashes of the deleted revisions.
    #[n(5)]
    pub(crate) revisions: SortedSet<RevisionHash>,
  }
}

pub(crate) mod missing {
  use super::*;

  /// Request body of `POST /api/missing`.
  #[derive(Debug, Encode, Decode)]
  pub(crate) struct Request {
    /// Hashes to check for.
    #[n(1)]
    pub(crate) hashes: SortedSet<Hash>,
  }

  /// Response body of `POST /api/missing`.
  #[derive(Encode, Decode)]
  pub(crate) struct Response {
    /// Hashes not present on the server.
    #[n(1)]
    pub(crate) hashes: SortedSet<Hash>,
  }
}

pub(crate) mod number {
  use super::*;

  /// Response body of `GET /api/number/{number}`.
  #[derive(Encode, Decode)]
  pub(crate) struct Response {
    /// Fingerprint of the package.
    #[n(1)]
    pub(crate) package: Fingerprint,
    /// Hash of the revision with the given number.
    #[n(2)]
    pub(crate) revision: RevisionHash,
  }
}

pub(crate) mod numbers {
  use super::*;

  /// Response body of `GET /api/numbers`.
  #[derive(Default, Encode, Decode)]
  pub(crate) struct Response {
    /// Package numbers present on the server.
    #[n(1)]
    pub(crate) numbers: SortedSet<u64>,
  }
}

pub(crate) mod packages {
  use super::*;

  /// Response body of `GET /api/packages`.
  #[derive(Default, Encode, Decode)]
  pub(crate) struct Response {
    /// Fingerprints of packages on the server.
    #[n(1)]
    pub(crate) packages: SortedSet<Fingerprint>,
  }
}

pub(crate) mod revision {
  use super::*;

  pub(crate) mod get {
    use super::*;

    /// Response body of `GET /api/revision/{revision}`.
    #[derive(Encode, Decode)]
    pub(crate) struct Response {
      /// Numbers of the packages whose head is the revision.
      #[n(1)]
      pub(crate) numbers: SortedSet<u64>,
    }
  }

  pub(crate) mod post {
    use super::*;

    /// How an uploaded package relates to existing packages.
    #[derive(Clone, Copy, Debug, Default, Encode, Decode)]
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

    impl Mode {
      pub(crate) fn verb(self) -> &'static str {
        match self {
          Self::New => "created",
          Self::Replace { .. } => "replaced",
          Self::Update { .. } => "updated",
        }
      }
    }

    /// Request body of `POST /api/revision/{revision}`.
    #[derive(Debug, Encode, Decode)]
    pub(crate) struct Request {
      /// How the uploaded package relates to existing packages.
      #[n(1)]
      pub(crate) mode: Mode,
    }

    /// Response body of `POST /api/revision/{revision}`.
    #[derive(Encode, Decode)]
    pub(crate) struct Response {
      /// Number assigned to the uploaded package.
      #[n(1)]
      pub(crate) number: u64,
    }
  }
}
