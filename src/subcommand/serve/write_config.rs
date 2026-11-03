use super::*;

pub(crate) enum WriteConfig {
  Admin { domain: Host, key: PublicKey },
  Forbidden,
  Unrestricted,
}
