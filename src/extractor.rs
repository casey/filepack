use {
  super::*,
  axum::extract::{Extension, Path, Query},
  axum_extra::TypedHeader,
};

pub(crate) trait Extractor {
  fn describe(endpoint: &mut reference::Endpoint);
}

impl Extractor for Body {
  fn describe(_endpoint: &mut reference::Endpoint) {}
}

impl<T> Extractor for Extension<T> {
  fn describe(_endpoint: &mut reference::Endpoint) {}
}

impl<T: Extractor> Extractor for Option<T> {
  fn describe(endpoint: &mut reference::Endpoint) {
    T::describe(endpoint);
  }
}

impl<T: PathExtractor> Extractor for Path<T> {
  fn describe(endpoint: &mut reference::Endpoint) {
    endpoint.parameters.extend(T::parameters());
  }
}

impl<T> Extractor for Query<T> {
  fn describe(_endpoint: &mut reference::Endpoint) {}
}

impl Extractor for TypedHeader<headers::Range> {
  fn describe(endpoint: &mut reference::Endpoint) {
    endpoint.range = true;
  }
}
