use super::*;

pub(crate) static DATE: LazyLock<Regex> =
  LazyLock::new(|| r"^\d\d\d\d-\d\d-\d\d$".parse().unwrap());

pub(crate) static KEY_NAME: LazyLock<Regex> =
  LazyLock::new(|| "^[a-z0-9]+(-[a-z0-9]+)*$".parse().unwrap());

pub(crate) static PUBLIC_KEY: LazyLock<Regex> =
  LazyLock::new(|| "^[A-Za-z0-9]{64}$".parse().unwrap());
