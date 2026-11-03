use super::*;

pub(crate) struct RedirectConfig {
  pub(crate) destination: Url,
  pub(crate) domains: HashSet<Host>,
}

impl RedirectConfig {
  pub(crate) fn with_path_and_query(&self, uri: &Uri) -> Url {
    let mut destination = self.destination.clone();
    destination.set_path(uri.path());
    destination.set_query(uri.query());
    destination
  }
}
