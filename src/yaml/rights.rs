use super::*;

/// Rights statement for a package's content.
#[skip_serializing_none]
#[derive(Debug, Deserialize, PartialEq, Json)]
#[serde(deny_unknown_fields)]
pub(crate) struct Rights {
  /// Reason the content is believed to have the stated status.
  #[json(example = "Published in 1929.")]
  pub(crate) basis: Option<Text>,
  /// License of the content.
  #[json(example = "CC-BY-4.0")]
  pub(crate) license: Option<License>,
  /// Links supporting the rights statement.
  #[json(example = "https://tobin-society.org/spirit-guide/copyright")]
  pub(crate) links: Option<NonEmptyOrderedSet<CheckedUrl>>,
  /// Notices required to accompany the content.
  #[json(example = "Courtesy of the New York Public Library")]
  pub(crate) notices: Option<NonEmptyOrderedSet<Text>>,
  /// Rights status of the content.
  #[json(example = "licensed")]
  pub(crate) status: crate::rights::Status,
  /// Path to the text of the license.
  #[json(example = "LICENSE.txt")]
  pub(crate) text: Option<RelativePath>,
}

impl Rights {
  pub(crate) fn load(self) -> Result<crate::Rights> {
    let Self {
      basis,
      license,
      links,
      notices,
      status,
      text,
    } = self;

    if status == crate::rights::Status::Licensed {
      ensure! {
        license.is_some() || text.is_some(),
        error::RightsLicenseMissing,
      }

      if let Some(license @ License::Reference(_)) = &license {
        ensure! {
          text.is_some(),
          error::RightsTextMissing { license: license.to_string() },
        }
      }
    } else {
      ensure! {
        license.is_none(),
        error::RightsLicenseStatus { status: status.to_string() },
      }

      ensure! {
        text.is_none(),
        error::RightsTextStatus { status: status.to_string() },
      }
    }

    Ok(crate::Rights {
      basis,
      license: license.map(|license| license.to_string().parse().unwrap()),
      links,
      notices,
      status,
      text,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn load_errors() {
    #[track_caller]
    fn case(
      license: Option<&str>,
      status: crate::rights::Status,
      text: Option<&str>,
      expected: &str,
    ) {
      assert_eq!(
        Rights {
          basis: None,
          license: license.map(|license| license.parse().unwrap()),
          links: None,
          notices: None,
          status,
          text: text.map(|text| text.parse().unwrap()),
        }
        .load()
        .unwrap_err()
        .to_string(),
        expected,
      );
    }

    case(
      None,
      crate::rights::Status::Licensed,
      None,
      "status `licensed` requires `rights.license` or `rights.text`",
    );

    case(
      Some("CC0-1.0"),
      crate::rights::Status::Unknown,
      None,
      "`rights.license` may not be used with status `unknown`",
    );

    case(
      Some("LicenseRef-foo"),
      crate::rights::Status::Licensed,
      None,
      "license `LicenseRef-foo` requires `rights.text`",
    );

    case(
      None,
      crate::rights::Status::Unknown,
      Some("foo"),
      "`rights.text` may not be used with status `unknown`",
    );
  }
}
