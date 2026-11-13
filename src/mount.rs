use super::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Mount {
  pub(crate) name: Option<String>,
  pub(crate) package: PackageIdentifier,
}

impl Display for Mount {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.package)?;

    if let Some(name) = &self.name {
      write!(f, ":{name}")?;
    }

    Ok(())
  }
}

impl FromStr for Mount {
  type Err = MountError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let (package, name) = if let Some((package, name)) = s.split_once(':') {
      (package, Some(name))
    } else {
      (s, None)
    };

    if let Some(name) = name {
      ensure!(re::MOUNT_NAME.is_match(name), mount_error::Name { name });
    }

    Ok(Self {
      name: name.map(str::to_owned),
      package: package.parse()?,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!("0".parse::<Mount>().unwrap().to_string(), "0");
    assert_eq!("0:foo".parse::<Mount>().unwrap().to_string(), "0:foo");
  }

  #[test]
  fn parse() {
    #[track_caller]
    fn ok(s: &str, package: PackageIdentifier, name: Option<&str>) {
      assert_eq!(
        s.parse::<Mount>().unwrap(),
        Mount {
          name: name.map(str::to_owned),
          package,
        },
      );
    }

    #[track_caller]
    fn err(s: &str, expected: &str) {
      assert_eq!(s.parse::<Mount>().unwrap_err().to_string(), expected);
    }

    let fingerprint = Fingerprint::EXAMPLE.parse().unwrap();

    ok("0", PackageIdentifier::Number(0), None);
    ok("0:foo-bar", PackageIdentifier::Number(0), Some("foo-bar"));
    ok(
      &format!("{}:foo", Fingerprint::EXAMPLE),
      PackageIdentifier::Fingerprint(fingerprint),
      Some("foo"),
    );

    err("0:Foo", "invalid mount name `Foo`");
    err("foo:bar", "unrecognized package identifier `foo`");
  }
}
