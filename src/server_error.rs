use super::*;

#[derive(Debug, Snafu)]
#[snafu(context(suffix(false)), visibility(pub(crate)))]
pub(crate) enum ServerError {
  #[snafu(display("I/O error at {path}"))]
  FilesystemIo {
    path: Utf8PathBuf,
    source: io::Error,
  },
}

impl ServerError {
  fn status_code(&self) -> StatusCode {
    match self {
      Self::FilesystemIo { .. } => StatusCode::INTERNAL_SERVER_ERROR,
    }
  }

  fn message(&self) -> &'static str {
    match self {
      Self::FilesystemIo { .. } => "error serving request: filesystem I/O error",
    }
  }
}

impl IntoResponse for ServerError {
  fn into_response(self) -> Response {
    (self.status_code(), self.message()).into_response()
  }
}
