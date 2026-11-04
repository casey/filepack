use super::*;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Domain(String);

impl Domain {
  pub(crate) fn as_str(&self) -> &str {
    &self.0
  }
}

impl FromStr for Domain {
  type Err = HostError;

  fn from_str(domain: &str) -> Result<Self, Self::Err> {
    ensure!(!domain.is_empty(), host_error::DomainEmpty);

    if let Some(character) = domain
      .chars()
      .find(|&character| !character.is_ascii_alphanumeric() && character != '-' && character != '.')
    {
      return host_error::DomainCharacter { character, domain }.fail();
    }

    ensure!(domain.len() <= 253, host_error::DomainLength { domain });

    for label in domain.split('.') {
      ensure!(!label.is_empty(), host_error::DomainLabelEmpty { domain });

      ensure!(
        label.len() <= 63,
        host_error::DomainLabelLength { domain, label },
      );

      ensure!(
        !label.starts_with('-') && !label.ends_with('-'),
        host_error::DomainLabelHyphen { domain, label },
      );
    }

    ensure!(
      !domain
        .rsplit('.')
        .next()
        .unwrap()
        .bytes()
        .all(|byte| byte.is_ascii_digit()),
      host_error::DomainNumeric { domain },
    );

    Ok(Self(domain.to_ascii_lowercase()))
  }
}

impl Display for Domain {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}
