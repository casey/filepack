use super::*;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Host {
  Domain(String),
  Ipv4(Ipv4Addr),
  Ipv6(Ipv6Addr),
}

impl FromStr for Host {
  type Err = HostError;

  fn from_str(host: &str) -> Result<Self, Self::Err> {
    if let Some(address) = host.strip_prefix('[') {
      return address
        .strip_suffix(']')
        .context(host_error::BracketedIpv6Unterminated { host })?
        .parse()
        .ok()
        .map(Self::Ipv6)
        .context(host_error::BracketedIpv6 { host });
    }

    if let Ok(address) = host.parse() {
      return Ok(Self::Ipv4(address));
    }

    if let Ok(address) = host.parse() {
      return Ok(Self::Ipv6(address));
    }

    ensure!(!host.is_empty(), host_error::DomainEmpty);

    if let Some(character) = host
      .chars()
      .find(|&character| !character.is_ascii_alphanumeric() && character != '-' && character != '.')
    {
      return host_error::DomainCharacter { character, host }.fail();
    }

    ensure!(host.len() <= 253, host_error::DomainLength { host });

    for label in host.split('.') {
      ensure!(!label.is_empty(), host_error::DomainLabelEmpty { host });

      ensure!(
        label.len() <= 63,
        host_error::DomainLabelLength { host, label },
      );

      ensure!(
        !label.starts_with('-') && !label.ends_with('-'),
        host_error::DomainLabelHyphen { host, label },
      );
    }

    ensure!(
      !host
        .rsplit('.')
        .next()
        .unwrap()
        .bytes()
        .all(|byte| byte.is_ascii_digit()),
      host_error::DomainNumeric { host },
    );

    Ok(Self::Domain(host.to_ascii_lowercase()))
  }
}

impl Display for Host {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Domain(domain) => write!(f, "{domain}"),
      Self::Ipv4(address) => write!(f, "{address}"),
      Self::Ipv6(address) => write!(f, "[{address}]"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    #[track_caller]
    fn case(host: &str, expected: &str) {
      assert_eq!(host.parse::<Host>().unwrap().to_string(), expected);
    }

    case("foo", "foo");
    case("FOO.bar", "foo.bar");
    case("127.0.0.1", "127.0.0.1");
    case("::1", "[::1]");
    case("[0:0::1]", "[::1]");
  }

  #[test]
  fn parse() {
    #[track_caller]
    fn ok(host: &str) {
      assert!(host.parse::<Host>().is_ok());
    }

    #[track_caller]
    fn err(host: &str, error: HostError) {
      assert_eq!(host.parse::<Host>().unwrap_err(), error);
    }

    let long = [
      "a".repeat(63),
      "b".repeat(63),
      "c".repeat(63),
      "d".repeat(61),
    ]
    .join(".");

    ok("foo-bar");
    ok("foo0");
    ok(&"a".repeat(63));
    ok(&long);

    err("", HostError::DomainEmpty);

    err(
      "foo_bar",
      HostError::DomainCharacter {
        character: '_',
        host: "foo_bar".into(),
      },
    );

    err(
      "foo.",
      HostError::DomainLabelEmpty {
        host: "foo.".into(),
      },
    );

    for host in ["-foo", "foo-"] {
      err(
        host,
        HostError::DomainLabelHyphen {
          host: host.into(),
          label: host.into(),
        },
      );
    }

    err(
      &"a".repeat(64),
      HostError::DomainLabelLength {
        host: "a".repeat(64),
        label: "a".repeat(64),
      },
    );

    err(
      &format!("{long}d"),
      HostError::DomainLength {
        host: format!("{long}d"),
      },
    );

    err("0", HostError::DomainNumeric { host: "0".into() });

    err(
      "[foo]",
      HostError::BracketedIpv6 {
        host: "[foo]".into(),
      },
    );

    err(
      "[::1",
      HostError::BracketedIpv6Unterminated {
        host: "[::1".into(),
      },
    );
  }
}
