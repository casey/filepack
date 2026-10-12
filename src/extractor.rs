use {
  super::*,
  axum::extract::{Extension, Path, Query},
  axum_extra::TypedHeader,
};

pub(crate) trait Extractor {
  fn annotate(endpoint: &mut Endpoint);
}

impl Extractor for Body {
  fn annotate(_endpoint: &mut Endpoint) {}
}

impl<T> Extractor for Extension<T> {
  fn annotate(_endpoint: &mut Endpoint) {}
}

impl<T: Extractor> Extractor for Option<T> {
  fn annotate(endpoint: &mut Endpoint) {
    T::annotate(endpoint);
  }
}

impl<T> Extractor for Path<T> {
  fn annotate(_endpoint: &mut Endpoint) {}
}

impl<T> Extractor for Query<T> {
  fn annotate(_endpoint: &mut Endpoint) {}
}

impl<T> Extractor for TypedHeader<T> {
  fn annotate(_endpoint: &mut Endpoint) {}
}
