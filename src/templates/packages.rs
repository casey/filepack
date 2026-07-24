use super::*;

#[derive(Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum View {
  Grid,
  #[default]
  List,
}

#[derive(Boilerplate)]
pub(crate) struct PackagesHtml {
  pub(crate) packages: Vec<(Fingerprint, Option<Metadata>)>,
  pub(crate) view: View,
}

impl PackagesHtml {
  fn packages(
    &self,
  ) -> impl Iterator<Item = (Fingerprint, Option<&Component>, Option<&Component>)> {
    self.packages.iter().map(|(fingerprint, metadata)| {
      (
        *fingerprint,
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
          <h1>Packages</h1>
          <ul class=grid>
            <li>
              <a href=/package/{fingerprint}>
                <img loading=lazy src=/artwork/{fingerprint} width=150 height=150>
              </a>
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
}
