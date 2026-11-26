use super::*;

#[derive(Boilerplate)]
pub(crate) struct HistoryHtml {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) identifier: PackageIdentifier,
  pub(crate) metadata: Option<Metadata>,
  pub(crate) revisions: Vec<(RevisionHash, BTreeSet<PublicKey>)>,
}

impl Page for HistoryHtml {
  fn open_graph_image(&self) -> Option<OpenGraphImage> {
    OpenGraphImage::artwork(self.metadata.as_ref()?, self.fingerprint)
  }

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
        fingerprint: Fingerprint::EXAMPLE.parse().unwrap(),
        identifier: PackageIdentifier::Number(0),
        metadata: None,
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
                <li>signed by <a href=/key/{key}><code>{key}</code></a></li>
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

  #[test]
  fn open_graph() {
    let html = HistoryHtml {
      fingerprint: Fingerprint::EXAMPLE.parse().unwrap(),
      identifier: PackageIdentifier::Number(0),
      metadata: Some(Metadata {
        artwork: Some(Image::test("foo.png")),
        ..default()
      }),
      revisions: Vec::new(),
    };

    assert_eq!(
      html.open_graph_image(),
      Some(OpenGraphImage {
        alt: "Artwork for Untitled package".into(),
        dimensions: Dimensions {
          height: 1,
          width: 1,
        },
        path: format!("artwork/{}", Fingerprint::EXAMPLE),
      }),
    );
  }
}
