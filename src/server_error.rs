use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum ServerError {
  #[snafu(display("invalid authorization token"))]
  AuthorizationInvalid { source: jsonwebtoken::errors::Error },
  #[snafu(display("malformed authorization header"))]
  AuthorizationMalformed,
  #[snafu(display("missing authorization header"))]
  AuthorizationMissing,
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
  #[snafu(display("file with hash {hash} not found"))]
  FileNotFound { hash: Hash, source: io::Error },
  #[snafu(display("I/O error at {path}"))]
  FilesystemIo {
    path: Utf8PathBuf,
    source: io::Error,
  },
  #[snafu(display("failed to decode metadata for package {hash}"))]
  PackageMetadataDecode { hash: Hash, source: DecodeError },
  #[snafu(display("package {fingerprint} not found"))]
  PackageNotFound { fingerprint: Fingerprint },
  #[snafu(display("package {fingerprint} root directory is unverified"))]
  PackageUnverified { fingerprint: Fingerprint },
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
      Self::AuthorizationInvalid { .. }
      | Self::AuthorizationMalformed
      | Self::AuthorizationMissing
      | Self::DirectoryDecode { .. }
      | Self::DirectoryFileMissing { .. }
      | Self::DirectoryNotFound { .. }
      | Self::DirectoryUnverified { .. }
      | Self::FileNotFound { .. }
      | Self::PackageMetadataDecode { .. }
      | Self::PackageNotFound { .. }
      | Self::PackageUnverified { .. }
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
      | Self::FilesystemIo { .. } => StatusCode::INTERNAL_SERVER_ERROR,
      Self::DirectoryDecode { .. }
      | Self::DirectoryFileMissing { .. }
      | Self::DirectoryUnverified { .. }
      | Self::PackageMetadataDecode { .. }
      | Self::PackageUnverified { .. }
      | Self::UploadBodyRead { .. }
      | Self::UploadHashMismatch { .. } => StatusCode::BAD_REQUEST,
      Self::DirectoryNotFound { .. }
      | Self::FileNotFound { .. }
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
