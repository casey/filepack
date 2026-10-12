use {
  super::*,
  axum::extract::{Extension, Path, Query},
  axum_extra::TypedHeader,
};

pub(crate) trait Extractor {
  fn describe(endpoint: &mut Endpoint);
}

impl Extractor for Body {
  fn describe(_endpoint: &mut Endpoint) {}
}

impl<T> Extractor for Extension<T> {
  fn describe(_endpoint: &mut Endpoint) {}
}

impl<T: Extractor> Extractor for Option<T> {
  fn describe(endpoint: &mut Endpoint) {
    T::describe(endpoint);
  }
}

impl<T> Extractor for Path<T> {
  fn describe(_endpoint: &mut Endpoint) {}
}

impl<T> Extractor for Query<T> {
  fn describe(_endpoint: &mut Endpoint) {}
}

impl<T> Extractor for TypedHeader<T> {
  fn describe(_endpoint: &mut Endpoint) {}
}
