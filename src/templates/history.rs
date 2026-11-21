use super::*;

#[derive(Boilerplate)]
pub(crate) struct HistoryHtml {
  pub(crate) identifier: PackageIdentifier,
  pub(crate) revisions: Vec<(RevisionHash, BTreeSet<PublicKey>)>,
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
        identifier: PackageIdentifier::Number(0),
        revisions: vec![
          (
            RevisionHash::EXAMPLE.parse().unwrap(),
            BTreeSet::from([PublicKey::EXAMPLE.parse().unwrap()]),
          ),
          (RevisionHash::ZERO, BTreeSet::new()),
        ],
      }
      .to_string(),
      unindent(&format!(
        "
          <h1>Package 0 history</h1>
          <ol reversed>
            <li>
              <a href=/package/{revision}><code>{revision}</code></a>
              <ul>
                <li>signed by <code>{key}</code></li>
              </ul>
            </li>
            <li>
              <a href=/package/{zero}><code>{zero}</code></a>
            </li>
          </ol>
        ",
        key = PublicKey::EXAMPLE,
        revision = RevisionHash::EXAMPLE,
        zero = RevisionHash::ZERO,
      )),
    );
  }
}
