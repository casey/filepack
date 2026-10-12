use {
  super::*,
  axum::{
    extract::{Extension, Path},
    response::Redirect,
  },
  axum_extra::{TypedHeader, either::Either},
};

pub(crate) trait Describe {
  fn describe(endpoint: &mut reference::Endpoint, schemas: &mut BTreeMap<String, Schema>);
}

impl Describe for () {
  fn describe(_endpoint: &mut reference::Endpoint, _schemas: &mut BTreeMap<String, Schema>) {}
}

impl Describe for Body {
  fn describe(endpoint: &mut reference::Endpoint, _schemas: &mut BTreeMap<String, Schema>) {
    endpoint.request = Some(reference::Request::Bytes);
  }
}

impl<T> Describe for Extension<T> {
  fn describe(_endpoint: &mut reference::Endpoint, _schemas: &mut BTreeMap<String, Schema>) {}
}

impl<T: Describe> Describe for Option<T> {
  fn describe(endpoint: &mut reference::Endpoint, schemas: &mut BTreeMap<String, Schema>) {
    T::describe(endpoint, schemas);
  }
}

impl<T: PathExtractor> Describe for Path<T> {
  fn describe(endpoint: &mut reference::Endpoint, _schemas: &mut BTreeMap<String, Schema>) {
    endpoint.parameters.extend(T::parameters());
  }
}

impl Describe for Redirect {
  fn describe(endpoint: &mut reference::Endpoint, _schemas: &mut BTreeMap<String, Schema>) {
    endpoint.response = Some(reference::Response::Redirect);
  }
}

impl<E1: Describe, E2> Describe for Either<E1, E2> {
  fn describe(endpoint: &mut reference::Endpoint, schemas: &mut BTreeMap<String, Schema>) {
    E1::describe(endpoint, schemas);
  }
}

impl<T: Describe, E> Describe for Result<T, E> {
  fn describe(endpoint: &mut reference::Endpoint, schemas: &mut BTreeMap<String, Schema>) {
    T::describe(endpoint, schemas);
  }
}

impl Describe for TypedHeader<headers::Range> {
  fn describe(endpoint: &mut reference::Endpoint, _schemas: &mut BTreeMap<String, Schema>) {
    endpoint.range = true;
  }
}
