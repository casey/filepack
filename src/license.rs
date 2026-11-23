use super::*;

/// License of a package's content.
///
/// One of `permission`, `public-domain`, `reserved`, `unknown`, or an SPDX
/// license identifier.
#[derive(Clone, Copy, Debug, DeserializeFromStr, Json, PartialEq)]
#[json(string)]
pub(crate) struct License(pub(crate) spdx::LicenseId);

impl Display for License {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0.name)
  }
}

impl FromStr for License {
  type Err = LicenseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let identifier = spdx::license_id(s)
      .filter(|identifier| identifier.name == s)
      .context(license_error::Unknown { license: s })?;

    ensure! {
      !identifier.is_deprecated(),
      license_error::Deprecated { license: s },
    }

    Ok(Self(identifier))
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

    err("foo", "unknown SPDX license identifier `foo`");
    err("cc-by-4.0", "unknown SPDX license identifier `cc-by-4.0`");
    err("CC-BY-4.0+", "unknown SPDX license identifier `CC-BY-4.0+`");
    err("GPL-2.0", "SPDX license identifier `GPL-2.0` is deprecated");
  }
}
