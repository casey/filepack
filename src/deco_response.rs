use super::*;

pub(crate) struct DecoResponse<T>(pub(crate) T);

impl<T: Encode> IntoResponse for DecoResponse<T> {
  fn into_response(self) -> Response {
    self.0.encode_to_vec().into_response()
  }
}

impl<T: Encode> Describe for DecoResponse<T> {
  fn describe(endpoint: &mut reference::Endpoint, registry: &mut deco::Registry) {
    let name = T::deco_schema(registry);

    let deco::Schema::Struct { fields, .. } = registry.remove(&name).unwrap() else {
      unreachable!()
    };

    endpoint.response = Some(reference::Response::Deco { fields });
  }
}
