use super::*;

pub(crate) struct ServerConfig {
  pub(crate) domains: HashSet<Domain>,
  pub(crate) mounts: Vec<Mount>,
  pub(crate) url: Option<Url>,
}

impl ServerConfig {
  pub(crate) fn mount(&self, name: &str) -> Option<&Mount> {
    self
      .mounts
      .iter()
      .find(|mount| mount.name.as_deref() == Some(name))
  }
}
