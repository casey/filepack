use super::*;

pub(crate) enum SpawnConfig {
  Http,
  Https,
  Redirect(Url),
}
