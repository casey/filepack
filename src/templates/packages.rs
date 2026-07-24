use super::*;

#[derive(Boilerplate)]
pub(crate) struct PackagesHtml {
  pub(crate) packages: Vec<(Fingerprint, Option<Metadata>)>,
  pub(crate) view: View,
}

impl PackagesHtml {
  fn packages(
    &self,
  ) -> impl Iterator<Item = (Fingerprint, bool, Option<&Component>, Option<&Component>)> {
    self.packages.iter().map(|(fingerprint, metadata)| {
      (
        *fingerprint,
        metadata
          .as_ref()
          .is_some_and(|metadata| metadata.artwork.is_some()),
        metadata
          .as_ref()
          .and_then(|metadata| metadata.creator.as_deref()),
        metadata
          .as_ref()
          .and_then(|metadata| metadata.title.as_deref()),
      )
    })
  }
}

impl Page for PackagesHtml {
  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/packages.css")
  }

  fn title(&self) -> String {
    "packages · filepack".into()
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn grid() {
    let fingerprint = test::FINGERPRINT.parse::<Fingerprint>().unwrap();

    let metadata = Metadata {
      artwork: Some("foo.png".parse().unwrap()),
      creator: Some("foo".parse().unwrap()),
      title: Some("bar".parse().unwrap()),
      ..default()
    };

    assert_eq!(
      PackagesHtml {
        packages: vec![(fingerprint, Some(metadata)), (fingerprint, None)],
        view: View::Grid,
      }
      .to_string(),
      unindent(&format!(
        "
          <header>
            <h1>Packages</h1>
            <nav>
              <a href=/packages>List</a> | <a>Grid</a>
            </nav>
          </header>
          <ul class=grid>
            <li>
              <a href=/package/{fingerprint}>
                <img loading=lazy src=/artwork/{fingerprint} width=150 height=150>
              </a>
              <span class=title>bar</span>
              <span class=creator>foo</span>
            </li>
            <li>
              <a href=/package/{fingerprint}></a>
            </li>
          </ul>
        ",
        fingerprint = test::FINGERPRINT,
      )),
    );
  }

  #[test]
  fn list() {
    let fingerprint = test::FINGERPRINT.parse::<Fingerprint>().unwrap();

    assert_eq!(
      PackagesHtml {
        packages: vec![(fingerprint, None)],
        view: View::List,
      }
      .to_string(),
      unindent(&format!(
        "
          <header>
            <h1>Packages</h1>
            <nav>
              <a>List</a> | <a href=/packages?view=grid>Grid</a>
            </nav>
          </header>
          <ul>
            <li>
              <a href=/package/{fingerprint}>
                <code>{fingerprint}</code>
              </a>
            </li>
          </ul>
        ",
        fingerprint = test::FINGERPRINT,
      )),
    );
  }
}
