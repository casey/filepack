use axum::{
  http::StatusCode,
  response::{IntoResponse, Response},
};

#[derive(Debug)]
pub(crate) enum ServerError {
  NotFound(String),
}

impl IntoResponse for ServerError {
  fn into_response(self) -> Response {
    match self {
      ServerError::NotFound(message) => (StatusCode::NOT_FOUND, message).into_response(),
    }
  }
}
