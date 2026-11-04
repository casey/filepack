use super::*;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Host {
  Domain(Domain),
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

    Ok(Self::Domain(host.parse()?))
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
        domain: "foo_bar".into(),
      },
    );

    err(
      "foo.",
      HostError::DomainLabelEmpty {
        domain: "foo.".into(),
      },
    );

    for host in ["-foo", "foo-"] {
      err(
        host,
        HostError::DomainLabelHyphen {
          domain: host.into(),
          label: host.into(),
        },
      );
    }

    err(
      &"a".repeat(64),
      HostError::DomainLabelLength {
        domain: "a".repeat(64),
        label: "a".repeat(64),
      },
    );

    err(
      &format!("{long}d"),
      HostError::DomainLength {
        domain: format!("{long}d"),
      },
    );

    err("0", HostError::DomainNumeric { domain: "0".into() });

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
