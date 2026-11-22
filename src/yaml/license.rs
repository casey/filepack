use super::*;

/// License of a package's content.
///
/// One of `permission`, `public-domain`, `reserved`, `unknown`, or an SPDX
/// license identifier.
#[derive(Clone, Copy, Debug, DeserializeFromStr, Json, PartialEq)]
#[json(string)]
pub(crate) enum License {
  Identifier(spdx::LicenseId),
  Keyword(Keyword),
}

#[derive(Clone, Copy, Debug, Display, EnumIter, EnumString, PartialEq)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum Keyword {
  Permission,
  PublicDomain,
  Reserved,
  Unknown,
}

impl Display for License {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Identifier(identifier) => write!(f, "{}", identifier.name),
      Self::Keyword(keyword) => write!(f, "{keyword}"),
    }
  }
}

impl FromStr for License {
  type Err = LicenseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if let Ok(keyword) = s.parse() {
      Ok(Self::Keyword(keyword))
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

    ok("permission");
    ok("public-domain");
    ok("reserved");
    ok("unknown");
    ok("CC-BY-4.0");

    err(
      "foo",
      "unknown license `foo`: expected `permission`, `public-domain`, `reserved`, `unknown`, or an \
       SPDX license identifier",
    );
    err(
      "cc-by-4.0",
      "unknown license `cc-by-4.0`: expected `permission`, `public-domain`, `reserved`, \
       `unknown`, or an SPDX license identifier",
    );
    err(
      "CC-BY-4.0+",
      "unknown license `CC-BY-4.0+`: expected `permission`, `public-domain`, `reserved`, \
       `unknown`, or an SPDX license identifier",
    );
    err("GPL-2.0", "SPDX license identifier `GPL-2.0` is deprecated");
  }

  #[test]
  fn keywords_are_not_identifiers() {
    assert_eq!(
      Keyword::iter()
        .map(|keyword| keyword.to_string())
        .filter(|keyword| {
          spdx::identifiers::LICENSES
            .iter()
            .any(|license| license.name.eq_ignore_ascii_case(keyword))
        })
        .collect::<Vec<String>>(),
      Vec::<String>::new(),
    );
  }
}
