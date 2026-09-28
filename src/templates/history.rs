use super::*;

#[derive(Boilerplate)]
pub(crate) struct HistoryHtml {
  pub(crate) identifier: PackageIdentifier,
  pub(crate) revisions: Vec<Revision>,
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
        identifier: PackageIdentifier::Number(1),
        revisions: vec![test::REVISION.parse().unwrap()],
      }
      .to_string(),
      unindent(&format!(
        "
          <h1>Package 1 history</h1>
          <ol reversed>
            <li><a href=/package/{revision}><code>{revision}</code></a></li>
          </ol>
        ",
        revision = test::REVISION,
      )),
    );
  }
}
