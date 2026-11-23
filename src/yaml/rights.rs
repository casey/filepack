use super::*;

/// Rights statement for a package's content.
#[skip_serializing_none]
#[derive(Debug, Deserialize, PartialEq, Json)]
#[serde(deny_unknown_fields)]
pub(crate) struct Rights {
  /// Reason the content is believed to have the stated license.
  #[json(example = "Published in 1929.")]
  pub(crate) basis: Option<Text>,
  /// Credit lines to display with the content.
  #[json(example = "Courtesy of the New York Public Library")]
  pub(crate) credits: Option<NonEmptyOrderedSet<Text>>,
  /// License of the content.
  #[json(example = "CC-BY-4.0")]
  pub(crate) license: Option<License>,
  /// Links supporting the basis.
  #[json(example = "https://tobin-society.org/spirit-guide/copyright")]
  pub(crate) links: Option<NonEmptyOrderedSet<CheckedUrl>>,
  /// Rights status of the content.
  #[json(example = "licensed")]
  pub(crate) status: crate::rights::Status,
}

impl Rights {
  pub(crate) fn load(self) -> Result<crate::Rights> {
    let Self {
      basis,
      credits,
      license,
      links,
      status,
    } = self;

    if license.is_some() {
      ensure! {
        status == crate::rights::Status::Licensed,
        error::RightsLicenseStatus { status: status.to_string() },
      }
    } else {
      ensure! {
        status != crate::rights::Status::Licensed,
        error::RightsLicenseMissing,
      }
    }

    Ok(crate::Rights {
      basis,
      credits,
      license: license.map(|license| license.to_string().parse().unwrap()),
      links,
      status,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn load_errors() {
    #[track_caller]
    fn case(license: Option<&str>, status: crate::rights::Status, expected: &str) {
      assert_eq!(
        Rights {
          basis: None,
          credits: None,
          license: license.map(|license| license.parse().unwrap()),
          links: None,
          status,
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
      "status `licensed` requires `rights.license`",
    );

    case(
      Some("CC0-1.0"),
      crate::rights::Status::Unknown,
      "`rights.license` may not be used with status `unknown`",
    );
  }
}
