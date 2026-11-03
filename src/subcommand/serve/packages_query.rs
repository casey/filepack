use super::*;

#[derive(Deserialize)]
pub(crate) struct PackagesQuery {
  pub(crate) order: Option<Order>,
  pub(crate) sort: Option<Sort>,
}

impl Describe for Query<PackagesQuery> {
  fn describe(endpoint: &mut crate::reference::Endpoint, _registry: &mut deco::Registry) {
    endpoint.query.extend([
      crate::reference::QueryParameter::new::<Order>("order"),
      crate::reference::QueryParameter::new::<Sort>("sort"),
    ]);
  }
}
