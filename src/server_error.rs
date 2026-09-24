use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub enum ServerError {
  #[snafu(display("package {fingerprint} artwork not found"))]
  ArtworkNotFound { fingerprint: Fingerprint },
  #[snafu(display("invalid authorization token"))]
  AuthorizationInvalid { source: AuthorizationError },
  #[snafu(display("malformed authorization header"))]
  AuthorizationMalformed,
  #[snafu(display("missing authorization header"))]
  AuthorizationMissing,
  #[snafu(context(false), display("failed to open database"))]
  Database { source: redb::DatabaseError },
  #[snafu(context(false), display("failed to commit database transaction"))]
  DatabaseCommit { source: redb::CommitError },
  #[snafu(context(false), display("database storage error"))]
  DatabaseStorage { source: redb::StorageError },
  #[snafu(context(false), display("failed to open database table"))]
  DatabaseTable { source: redb::TableError },
  #[snafu(context(false), display("failed to begin database transaction"))]
  DatabaseTransaction { source: redb::TransactionError },
  #[snafu(display("failed to read request body"))]
  DecoBody { source: axum::Error },
  #[snafu(display("failed to decode request body"))]
  DecoDecode { source: DecodeError },
  #[snafu(display("stored directory {hash} failed to decode"))]
  DirectoryCorrupt { hash: Hash, source: DecodeError },
  #[snafu(display("failed to decode directory {hash}"))]
  DirectoryDecode { hash: Hash, source: DecodeError },
  #[snafu(display(
    "directory {directory} references missing {ty} entry `{name}` with hash {hash}"
  ))]
  DirectoryEntryMissing {
    directory: Hash,
    hash: Hash,
    name: ComponentBuf,
    ty: EntryType,
  },
  #[snafu(display(
    "directory {directory} entry `{entry}` size mismatch, expected {expected} but found {actual}",
  ))]
  DirectoryEntrySizeMismatch {
    actual: u64,
    directory: Hash,
    entry: ComponentBuf,
    expected: u64,
  },
  #[snafu(display("directory {directory} entry `{entry}` totals error: {source}"))]
  DirectoryEntryTotals {
    directory: Hash,
    entry: ComponentBuf,
    source: TotalsError,
  },
  #[snafu(display("directory {hash} not found"))]
  DirectoryNotFound { hash: Hash },
  #[snafu(display("directory {hash} totals error"))]
  DirectoryTotals { hash: Hash, source: TotalsError },
  #[snafu(display("directory {directory} references unverified subdirectory {subdirectory}"))]
  DirectoryUnverified { directory: Hash, subdirectory: Hash },
  #[snafu(display("I/O error on file {hash}"))]
  FileIo { hash: Hash, source: io::Error },
  #[snafu(display("file with hash {hash} not found"))]
  FileNotFound { hash: Hash, source: io::Error },
  #[snafu(context(false), display("filesystem I/O error"))]
  Filesystem { source: FilesystemError },
  #[snafu(display("{source}"))]
  FingerprintParse { source: HexError },
  #[snafu(display("response invalid"))]
  InvalidResponse { source: http::Error },
  #[snafu(display(
    "{} {index} does not exist, package {fingerprint} has {}",
    ty.item_noun(),
    Count::new(*count, ty.item_noun()),
  ))]
  MediaItemDoesNotExist {
    count: usize,
    fingerprint: Fingerprint,
    index: Ordinal,
    ty: crate::MediaType,
  },
  #[snafu(display("expected media type {expected} but package {fingerprint} is {actual}"))]
  MediaType {
    fingerprint: Fingerprint,
    actual: crate::MediaType,
    expected: crate::MediaType,
  },
  #[snafu(display("media type {ty} does not have items"))]
  MediaTypeDoesNotHaveItems { ty: crate::MediaType },
  #[snafu(display("file `{path}` missing from package {fingerprint}"))]
  PackageFileMissing {
    fingerprint: Fingerprint,
    path: RelativePath,
  },
  #[snafu(display("file `{path}` not found in package {fingerprint}"))]
  PackageFileNotFound {
    fingerprint: Fingerprint,
    path: RelativePath,
  },
  #[snafu(display("package {fingerprint} not found"))]
  PackageFingerprintNotFound { fingerprint: Fingerprint },
  #[snafu(display("package {fingerprint} does not have media metadata"))]
  PackageMediaMetadataNotFound { fingerprint: Fingerprint },
  #[snafu(display("stored metadata for package {fingerprint} failed to decode"))]
  PackageMetadataCorrupt {
    fingerprint: Fingerprint,
    source: DecodeError,
  },
  #[snafu(display("failed to decode metadata for package {fingerprint}"))]
  PackageMetadataDecode {
    fingerprint: Fingerprint,
    source: DecodeError,
  },
  #[snafu(display("package {fingerprint} metadata references missing file `{path}`"))]
  PackageMetadataFileMissing {
    fingerprint: Fingerprint,
    path: RelativePath,
  },
  #[snafu(display("package {fingerprint} does not have metadata"))]
  PackageMetadataNotFound { fingerprint: Fingerprint },
  #[snafu(display("package {fingerprint} not mounted"))]
  PackageNotMounted { fingerprint: Fingerprint },
  #[snafu(display("package {fingerprint} already has number {number}"))]
  PackageNumberConflict {
    fingerprint: Fingerprint,
    number: u64,
  },
  #[snafu(display("package number {number} not found"))]
  PackageNumberNotFound { number: u64 },
  #[snafu(display("package {fingerprint} root directory is unverified"))]
  PackageRootUnverified { fingerprint: Fingerprint },
  #[snafu(display("page not found"))]
  PageNotFound,
  #[snafu(display("video {index} in package {fingerprint} does not have a placeholder image"))]
  PlaceholderNotFound {
    fingerprint: Fingerprint,
    index: Ordinal,
  },
  #[snafu(display("stored revision {revision} failed to decode"))]
  RevisionCorrupt {
    revision: Revision,
    source: DecodeError,
  },
  #[snafu(display("failed to get current time"))]
  Time { source: SystemTimeError },
  #[snafu(display("error reading body of upload with hash {hash}"))]
  UploadBodyRead { hash: Hash, source: axum::Error },
  #[snafu(display("expected upload with hash {expected} but got {actual}"))]
  UploadHashMismatch { actual: Hash, expected: Hash },
  #[snafu(display("writes forbidden"))]
  WriteForbidden,
}

impl ServerError {
  pub(crate) fn status_code(&self) -> StatusCode {
    match self {
      Self::AuthorizationInvalid { .. }
      | Self::AuthorizationMalformed
      | Self::AuthorizationMissing => StatusCode::UNAUTHORIZED,
      Self::Database { .. }
      | Self::DatabaseCommit { .. }
      | Self::DatabaseStorage { .. }
      | Self::DatabaseTable { .. }
      | Self::DatabaseTransaction { .. }
      | Self::FileIo { .. }
      | Self::Filesystem { .. }
      | Self::InvalidResponse { .. }
      | Self::PackageFileMissing { .. }
      | Self::DirectoryCorrupt { .. }
      | Self::PackageMetadataCorrupt { .. }
      | Self::RevisionCorrupt { .. }
      | Self::Time { .. } => StatusCode::INTERNAL_SERVER_ERROR,
      Self::DecoBody { .. }
      | Self::DecoDecode { .. }
      | Self::DirectoryDecode { .. }
      | Self::DirectoryEntryMissing { .. }
      | Self::DirectoryEntrySizeMismatch { .. }
      | Self::DirectoryEntryTotals { .. }
      | Self::DirectoryTotals { .. }
      | Self::DirectoryUnverified { .. }
      | Self::FingerprintParse { .. }
      | Self::PackageMetadataDecode { .. }
      | Self::PackageMetadataFileMissing { .. }
      | Self::PackageRootUnverified { .. }
      | Self::UploadBodyRead { .. }
      | Self::UploadHashMismatch { .. } => StatusCode::BAD_REQUEST,
      Self::ArtworkNotFound { .. }
      | Self::DirectoryNotFound { .. }
      | Self::FileNotFound { .. }
      | Self::MediaItemDoesNotExist { .. }
      | Self::MediaType { .. }
      | Self::MediaTypeDoesNotHaveItems { .. }
      | Self::PackageFileNotFound { .. }
      | Self::PackageFingerprintNotFound { .. }
      | Self::PackageMediaMetadataNotFound { .. }
      | Self::PackageMetadataNotFound { .. }
      | Self::PackageNotMounted { .. }
      | Self::PackageNumberNotFound { .. }
      | Self::PageNotFound
      | Self::PlaceholderNotFound { .. } => StatusCode::NOT_FOUND,
      Self::PackageNumberConflict { .. } => StatusCode::CONFLICT,
      Self::WriteForbidden => StatusCode::FORBIDDEN,
    }
  }
}

impl IntoResponse for ServerError {
  fn into_response(self) -> Response {
    (self.status_code(), self.to_string()).into_response()
  }
}
