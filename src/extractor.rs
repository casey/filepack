use {
  super::*,
  axum::extract::{Extension, Path, Query},
  axum_extra::TypedHeader,
};

pub(crate) trait Extractor {
  fn describe(endpoint: &mut reference::Endpoint, schemas: &mut BTreeMap<String, Schema>);
}

impl Extractor for Body {
  fn describe(endpoint: &mut reference::Endpoint, _schemas: &mut BTreeMap<String, Schema>) {
    endpoint.request = Some(reference::Request::Bytes);
  }
}

impl<T> Extractor for Extension<T> {
  fn describe(_endpoint: &mut reference::Endpoint, _schemas: &mut BTreeMap<String, Schema>) {}
}

impl<T: Extractor> Extractor for Option<T> {
  fn describe(endpoint: &mut reference::Endpoint, schemas: &mut BTreeMap<String, Schema>) {
    T::describe(endpoint, schemas);
  }
}

impl<T: PathExtractor> Extractor for Path<T> {
  fn describe(endpoint: &mut reference::Endpoint, _schemas: &mut BTreeMap<String, Schema>) {
    endpoint.parameters.extend(T::parameters());
  }
}

impl<T> Extractor for Query<T> {
  fn describe(_endpoint: &mut reference::Endpoint, _schemas: &mut BTreeMap<String, Schema>) {}
}

impl Extractor for TypedHeader<headers::Range> {
  fn describe(endpoint: &mut reference::Endpoint, _schemas: &mut BTreeMap<String, Schema>) {
    endpoint.range = true;
  }
}
