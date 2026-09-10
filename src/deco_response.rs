use super::*;

pub(crate) struct DecoResponse<T>(pub(crate) T);

impl<T: Encode> IntoResponse for DecoResponse<T> {
  fn into_response(self) -> Response {
    self.0.encode_to_vec().into_response()
  }
}
