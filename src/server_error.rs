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
  #[snafu(display("file with hash {hash} not found"))]
  FileNotFound { hash: Hash, source: io::Error },
  #[snafu(display("I/O error at {path}"))]
  FilesystemIo {
    path: Utf8PathBuf,
    source: io::Error,
  },
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
      | Self::FileNotFound { .. }
      | Self::UploadBodyRead { .. }
      | Self::UploadForbidden
      | Self::UploadHashMismatch { .. } => self.to_string(),
      Self::FilesystemIo { .. } => "filesystem I/O error".into(),
    }
  }

  fn status_code(&self) -> StatusCode {
    match self {
      Self::AuthorizationInvalid { .. }
      | Self::AuthorizationMalformed
      | Self::AuthorizationMissing => StatusCode::UNAUTHORIZED,
      Self::FileNotFound { .. } => StatusCode::NOT_FOUND,
      Self::FilesystemIo { .. } => StatusCode::INTERNAL_SERVER_ERROR,
      Self::UploadForbidden => StatusCode::FORBIDDEN,
      Self::UploadBodyRead { .. } | Self::UploadHashMismatch { .. } => StatusCode::BAD_REQUEST,
    }
  }
}

impl IntoResponse for ServerError {
  fn into_response(self) -> Response {
    (self.status_code(), self.message()).into_response()
  }
}
