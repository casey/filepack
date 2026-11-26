use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum PackageText {
  Colophon,
  License(usize),
  Readme,
}

impl PackageText {
  pub(crate) fn download(self, fingerprint: Fingerprint) -> String {
    match self {
      Self::Colophon => format!("/colophon/{fingerprint}"),
      Self::License(index) => format!("/license/{fingerprint}/{}", Ordinal(index)),
      Self::Readme => format!("/readme/{fingerprint}"),
    }
  }

  pub(crate) fn path(self, identifier: PackageIdentifier) -> String {
    match self {
      Self::Colophon => format!("/package/{identifier}/colophon"),
      Self::License(index) => format!("/package/{identifier}/license/{}", Ordinal(index)),
      Self::Readme => format!("/package/{identifier}/readme"),
    }
  }

  pub(crate) fn stem(self) -> Stem {
    match self {
      Self::Colophon => Stem::Colophon,
      Self::License(_) => Stem::License,
      Self::Readme => Stem::Readme,
    }
  }

  pub(crate) fn text(self, metadata: &Metadata, fingerprint: Fingerprint) -> ServerResult<&Text> {
    match self {
      Self::Colophon => metadata
        .package
        .as_ref()
        .and_then(|package| package.colophon.as_ref())
        .context(server_error::ColophonNotFound { fingerprint }),
      Self::License(index) => metadata
        .rights
        .as_ref()
        .and_then(|rights| rights.texts.as_ref())
        .and_then(|texts| texts.get(index))
        .context(server_error::LicenseTextNotFound { fingerprint, index }),
      Self::Readme => metadata
        .readme
        .as_ref()
        .context(server_error::ReadmeNotFound { fingerprint }),
    }
  }
}

impl Display for PackageText {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Colophon => write!(f, "Colophon"),
      Self::License(index) => write!(f, "License {}", Ordinal(*index)),
      Self::Readme => write!(f, "Readme"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(PackageText::Colophon.to_string(), "Colophon");
    assert_eq!(PackageText::License(1).to_string(), "License 2");
    assert_eq!(PackageText::Readme.to_string(), "Readme");
  }

  #[test]
  fn download() {
    let fingerprint = Fingerprint::EXAMPLE.parse().unwrap();

    assert_eq!(
      PackageText::Colophon.download(fingerprint),
      format!("/colophon/{fingerprint}"),
    );
    assert_eq!(
      PackageText::License(1).download(fingerprint),
      format!("/license/{fingerprint}/2"),
    );
    assert_eq!(
      PackageText::Readme.download(fingerprint),
      format!("/readme/{fingerprint}"),
    );
  }

  #[test]
  fn path() {
    let identifier = PackageIdentifier::Number(0);

    assert_eq!(
      PackageText::Colophon.path(identifier),
      "/package/0/colophon"
    );
    assert_eq!(
      PackageText::License(1).path(identifier),
      "/package/0/license/2"
    );
    assert_eq!(PackageText::Readme.path(identifier), "/package/0/readme");
  }
}
