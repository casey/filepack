use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum ServerError {
  #[snafu(display("file with hash {hash} not found"))]
  FileNotFound { hash: Hash, source: io::Error },
  #[snafu(display("I/O error at {path}"))]
  FilesystemIo {
    path: Utf8PathBuf,
    source: io::Error,
  },
  #[snafu(display("invalid upload JWT"))]
  UploadAuthInvalid { source: jsonwebtoken::errors::Error },
  #[snafu(display("malformed Authorization header"))]
  UploadAuthMalformed,
  #[snafu(display("missing Authorization header"))]
  UploadAuthMissing,
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
      Self::FilesystemIo { .. } => "filesystem I/O error".into(),
      Self::UploadAuthInvalid { .. } | Self::UploadAuthMalformed | Self::UploadAuthMissing => {
        "unauthorized".into()
      }
      Self::FileNotFound { .. }
      | Self::UploadBodyRead { .. }
      | Self::UploadForbidden
      | Self::UploadHashMismatch { .. } => self.to_string(),
    }
  }

  fn status_code(&self) -> StatusCode {
    match self {
      Self::FileNotFound { .. } => StatusCode::NOT_FOUND,
      Self::FilesystemIo { .. } => StatusCode::INTERNAL_SERVER_ERROR,
      Self::UploadAuthInvalid { .. } | Self::UploadAuthMalformed | Self::UploadAuthMissing => {
        StatusCode::UNAUTHORIZED
      }
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
