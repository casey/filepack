use super::*;

#[derive(Boilerplate)]
pub(crate) struct HistoryHtml {
  pub(crate) entries: Vec<(Revision, Fingerprint)>,
  pub(crate) identifier: PackageIdentifier,
}

impl Page for HistoryHtml {
  fn title(&self) -> String {
    format!("Package {} history · Filepack", self.identifier)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn history() {
    assert_eq!(
      HistoryHtml {
        entries: vec![(
          test::REVISION.parse().unwrap(),
          test::FINGERPRINT.parse().unwrap(),
        )],
        identifier: PackageIdentifier::Number(1),
      }
      .to_string(),
      unindent(&format!(
        "
          <h1>Package 1 history</h1>
          <ol reversed>
            <li>
              <a href=/package/{revision}><code>{revision}</code></a>
              <a href=/package/{fingerprint}><code>{fingerprint}</code></a>
            </li>
          </ol>
        ",
        fingerprint = test::FINGERPRINT,
        revision = test::REVISION,
      )),
    );
  }
}
