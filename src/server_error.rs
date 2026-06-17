use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum ServerError {
  #[snafu(display("package {fingerprint} artwork not found"))]
  ArtworkNotFound { fingerprint: Fingerprint },
  #[snafu(display("invalid authorization token"))]
  AuthorizationInvalid { source: jsonwebtoken::errors::Error },
  #[snafu(display("malformed authorization header"))]
  AuthorizationMalformed,
  #[snafu(display("missing authorization header"))]
  AuthorizationMissing,
  #[snafu(display("failed to read request body"))]
  CborBody { source: axum::Error },
  #[snafu(display("failed to decode request body"))]
  CborDecode { source: DecodeError },
  #[snafu(transparent)]
  Database { source: redb::DatabaseError },
  #[snafu(transparent)]
  DatabaseCommit { source: redb::CommitError },
  #[snafu(transparent)]
  DatabaseStorage { source: redb::StorageError },
  #[snafu(transparent)]
  DatabaseTable { source: redb::TableError },
  #[snafu(transparent)]
  DatabaseTransaction { source: redb::TransactionError },
  #[snafu(display("failed to decode directory {hash}"))]
  DirectoryDecode { hash: Hash, source: DecodeError },
  #[snafu(display("directory {directory} references missing file {file}"))]
  DirectoryFileMissing { directory: Hash, file: Hash },
  #[snafu(display("directory {hash} not found"))]
  DirectoryNotFound { hash: Hash },
  #[snafu(display("directory {directory} references unverified subdirectory {subdirectory}"))]
  DirectoryUnverified { directory: Hash, subdirectory: Hash },
  #[snafu(display("I/O error on file {hash}"))]
  FileIo { hash: Hash, source: io::Error },
  #[snafu(display("file with hash {hash} not found"))]
  FileNotFound { hash: Hash, source: io::Error },
  #[snafu(display("I/O error at {path}"))]
  FilesystemIo {
    path: Utf8PathBuf,
    source: io::Error,
  },
  #[snafu(display("failed to parse package fingerprint"))]
  FingerprintParse { source: Bech32Error },
  #[snafu(display("response invalid"))]
  InvalidResponse { source: http::Error },
  #[snafu(display(
    "{} {index} does not exist, package {fingerprint} has {}",
    ty.noun(),
    Count(*count, ty.noun()),
  ))]
  MediaItemDoesNotExist {
    fingerprint: Fingerprint,
    ty: crate::MediaType,
    index: Ordinal,
    count: usize,
  },
  #[snafu(display("expected media type {expected} but package {fingerprint} is {actual}"))]
  MediaType {
    fingerprint: Fingerprint,
    actual: crate::MediaType,
    expected: crate::MediaType,
  },
  #[snafu(display("file `{path}` missing from package {fingerprint}"))]
  PackageFileMissing {
    fingerprint: Fingerprint,
    path: RelativePath,
  },
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
  #[snafu(display("package {fingerprint} not found"))]
  PackageNotFound { fingerprint: Fingerprint },
  #[snafu(display("package {fingerprint} root directory is unverified"))]
  PackageRootUnverified { fingerprint: Fingerprint },
  #[snafu(display("page not found"))]
  PageNotFound,
  #[snafu(display("error reading body of upload with hash {hash}"))]
  UploadBodyRead { hash: Hash, source: axum::Error },
  #[snafu(display("uploads forbidden"))]
  UploadForbidden,
  #[snafu(display("expected upload with hash {expected} but got {actual}"))]
  UploadHashMismatch { actual: Hash, expected: Hash },
}

impl ServerError {
  fn message(&self) -> String {
    match self {
      Self::ArtworkNotFound { .. }
      | Self::AuthorizationInvalid { .. }
      | Self::AuthorizationMalformed
      | Self::AuthorizationMissing
      | Self::CborBody { .. }
      | Self::CborDecode { .. }
      | Self::DirectoryDecode { .. }
      | Self::DirectoryFileMissing { .. }
      | Self::DirectoryNotFound { .. }
      | Self::DirectoryUnverified { .. }
      | Self::FileIo { .. }
      | Self::FileNotFound { .. }
      | Self::FingerprintParse { .. }
      | Self::InvalidResponse { .. }
      | Self::MediaItemDoesNotExist { .. }
      | Self::MediaType { .. }
      | Self::PackageFileMissing { .. }
      | Self::PackageMediaMetadataNotFound { .. }
      | Self::PackageMetadataCorrupt { .. }
      | Self::PackageMetadataDecode { .. }
      | Self::PackageMetadataFileMissing { .. }
      | Self::PackageMetadataNotFound { .. }
      | Self::PackageNotFound { .. }
      | Self::PackageRootUnverified { .. }
      | Self::PageNotFound
      | Self::UploadBodyRead { .. }
      | Self::UploadForbidden
      | Self::UploadHashMismatch { .. } => self.to_string(),
      Self::Database { .. }
      | Self::DatabaseCommit { .. }
      | Self::DatabaseStorage { .. }
      | Self::DatabaseTable { .. }
      | Self::DatabaseTransaction { .. } => "database error".into(),
      Self::FilesystemIo { .. } => "filesystem I/O error".into(),
    }
  }

  fn status_code(&self) -> StatusCode {
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
      | Self::FilesystemIo { .. }
      | Self::InvalidResponse { .. }
      | Self::PackageFileMissing { .. }
      | Self::PackageMetadataCorrupt { .. } => StatusCode::INTERNAL_SERVER_ERROR,
      Self::CborBody { .. }
      | Self::CborDecode { .. }
      | Self::DirectoryDecode { .. }
      | Self::DirectoryFileMissing { .. }
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
      | Self::PackageMediaMetadataNotFound { .. }
      | Self::PackageMetadataNotFound { .. }
      | Self::PackageNotFound { .. }
      | Self::PageNotFound => StatusCode::NOT_FOUND,
      Self::UploadForbidden => StatusCode::FORBIDDEN,
    }
  }
}

impl IntoResponse for ServerError {
  fn into_response(self) -> Response {
    (self.status_code(), self.message()).into_response()
  }
}
