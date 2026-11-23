use super::*;

/// License of a package's content.
///
/// An SPDX license identifier or reference.
#[derive(Clone, Debug, DeserializeFromStr, Json, PartialEq)]
#[json(string)]
pub(crate) enum License {
  Identifier(spdx::LicenseId),
  Reference(String),
}

impl License {
  const REFERENCE_PREFIX: &'static str = "LicenseRef-";
}

impl Display for License {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Identifier(identifier) => write!(f, "{}", identifier.name),
      Self::Reference(name) => write!(f, "{}{name}", Self::REFERENCE_PREFIX),
    }
  }
}

impl FromStr for License {
  type Err = LicenseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if let Some(name) = s.strip_prefix(Self::REFERENCE_PREFIX) {
      ensure! {
        !name.is_empty()
          && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.')),
        license_error::Reference { license: s },
      }

      Ok(Self::Reference(name.into()))
    } else {
      let identifier = spdx::license_id(s)
        .filter(|identifier| identifier.name == s)
        .context(license_error::Unknown { license: s })?;

      ensure! {
        !identifier.is_deprecated(),
        license_error::Deprecated { license: s },
      }

      Ok(Self::Identifier(identifier))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_str() {
    #[track_caller]
    fn err(license: &str, expected: &str) {
      assert_eq!(
        license.parse::<License>().unwrap_err().to_string(),
        expected,
      );
    }

    #[track_caller]
    fn ok(license: &str) {
      assert_eq!(license.parse::<License>().unwrap().to_string(), license);
    }

    ok("CC-BY-4.0");
    ok("LicenseRef-foo");

    err("foo", "unknown SPDX license identifier `foo`");
    err("LicenseRef-", "invalid license reference `LicenseRef-`");
    err(
      "LicenseRef-foo_bar",
      "invalid license reference `LicenseRef-foo_bar`",
    );
    err("cc-by-4.0", "unknown SPDX license identifier `cc-by-4.0`");
    err("CC-BY-4.0+", "unknown SPDX license identifier `CC-BY-4.0+`");
    err("GPL-2.0", "SPDX license identifier `GPL-2.0` is deprecated");
  }
}
