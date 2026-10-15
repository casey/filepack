use super::*;

pub(crate) struct DecoResponse<T>(pub(crate) T);

impl<T: Encode> IntoResponse for DecoResponse<T> {
  fn into_response(self) -> Response {
    self.0.encode_to_vec().into_response()
  }
}

impl<T: Encode> Describe for DecoResponse<T> {
  fn describe(endpoint: &mut reference::Endpoint, schemas: &mut deco::Registry) {
    let name = T::schema(schemas);

    let schema = schemas.remove(&name).unwrap();

    endpoint.response = Some(reference::Response::Deco { schema });
  }
}
