use super::*;

/// Rights statement for a package's content.
#[skip_serializing_none]
#[derive(Clone, Debug, Decode, Encode, Json, PartialEq, Serialize)]
pub(crate) struct Rights {
  /// Reason the content is believed to have the stated license.
  #[n(1)]
  pub(crate) basis: Option<Text>,
  /// License of the content.
  #[n(2)]
  pub(crate) license: Text,
}

impl Rights {
  pub(crate) fn info(&self) -> Info {
    InfoBuilder::new()
      .value("license", &self.license)
      .optional("basis", self.basis.as_ref())
      .build()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn encoding() {
    assert_encoding(Rights {
      basis: Some("foo".parse().unwrap()),
      license: "unknown".parse().unwrap(),
    });
  }
}
