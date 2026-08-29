use super::*;

pub(crate) struct PageError(ServerError);

impl From<ServerError> for PageError {
  fn from(error: ServerError) -> Self {
    Self(error)
  }
}

impl IntoResponse for PageError {
  fn into_response(self) -> Response {
    let status = self.0.status_code();

    (
      status,
      ErrorHtml {
        message: self.0.message(),
        status,
      }
      .page(None),
    )
      .into_response()
  }
}
