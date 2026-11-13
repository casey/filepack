use super::*;

pub(crate) enum WriteConfig {
  Admin { domain: Domain, key: PublicKey },
  Forbidden,
  Unrestricted,
}
