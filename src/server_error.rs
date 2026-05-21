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
  #[snafu(display("failed to decode directory {hash}"))]
  DirectoryDecode { hash: Hash, source: DecodeError },
  #[snafu(display("directory {directory} references missing file {file}"))]
  DirectoryFileMissing { directory: Hash, file: Hash },
  #[snafu(display("directory {directory} references unverified subdirectory {subdirectory}"))]
  DirectorySubdirectoryUnverified { directory: Hash, subdirectory: Hash },
  #[snafu(display("file with hash {hash} not found"))]
  FileNotFound { hash: Hash, source: io::Error },
  #[snafu(display("I/O error at {path}"))]
  FilesystemIo {
    path: Utf8PathBuf,
    source: io::Error,
  },
  #[snafu(display("page not found"))]
  PageNotFound,
  #[snafu(transparent)]
  Redb { source: redb::Error },
  #[snafu(transparent)]
  RedbCommit { source: redb::CommitError },
  #[snafu(transparent)]
  RedbDatabase { source: redb::DatabaseError },
  #[snafu(transparent)]
  RedbStorage { source: redb::StorageError },
  #[snafu(transparent)]
  RedbTable { source: redb::TableError },
  #[snafu(transparent)]
  RedbTransaction { source: redb::TransactionError },
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
      | Self::DirectorySubdirectoryUnverified { .. }
      | Self::FileNotFound { .. }
      | Self::PageNotFound
      | Self::UploadBodyRead { .. }
      | Self::UploadForbidden
      | Self::UploadHashMismatch { .. } => self.to_string(),
      Self::Redb { .. }
      | Self::RedbCommit { .. }
      | Self::RedbDatabase { .. }
      | Self::RedbStorage { .. }
      | Self::RedbTable { .. }
      | Self::RedbTransaction { .. } => "database error".into(),
      Self::FilesystemIo { .. } => "filesystem I/O error".into(),
    }
  }

  fn status_code(&self) -> StatusCode {
    match self {
      Self::AuthorizationInvalid { .. }
      | Self::AuthorizationMalformed
      | Self::AuthorizationMissing => StatusCode::UNAUTHORIZED,
      Self::FilesystemIo { .. }
      | Self::Redb { .. }
      | Self::RedbCommit { .. }
      | Self::RedbDatabase { .. }
      | Self::RedbStorage { .. }
      | Self::RedbTable { .. }
      | Self::RedbTransaction { .. } => StatusCode::INTERNAL_SERVER_ERROR,
      Self::DirectoryDecode { .. }
      | Self::DirectoryFileMissing { .. }
      | Self::DirectorySubdirectoryUnverified { .. }
      | Self::UploadBodyRead { .. }
      | Self::UploadHashMismatch { .. } => StatusCode::BAD_REQUEST,
      Self::FileNotFound { .. } | Self::PageNotFound => StatusCode::NOT_FOUND,
      Self::UploadForbidden => StatusCode::FORBIDDEN,
    }
  }
}

impl IntoResponse for ServerError {
  fn into_response(self) -> Response {
    (self.status_code(), self.message()).into_response()
  }
}
