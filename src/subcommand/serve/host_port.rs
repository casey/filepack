use super::*;

#[derive(Clone, PartialEq)]
pub(crate) struct HostPort {
  pub(crate) host: Host,
  pub(crate) port: Option<u16>,
}
