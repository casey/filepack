use super::*;

pub(crate) static KEY_NAME: LazyLock<Regex> =
  LazyLock::new(|| Regex::new("^[a-z0-9]+(-[a-z0-9]+)*$").unwrap());

pub(crate) static PUBLIC_KEY: LazyLock<Regex> =
  LazyLock::new(|| Regex::new("^[A-Za-z0-9]{64}$").unwrap());
