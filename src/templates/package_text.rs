use super::*;

#[derive(Boilerplate)]
pub(crate) struct PackageTextHtml {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) identifier: PackageIdentifier,
  pub(crate) metadata: Metadata,
  pub(crate) text: String,
  pub(crate) ty: PackageText,
}

impl PackageTextHtml {
  fn footer(&self) -> FooterHtml {
    FooterHtml {
      download: self.ty.download(self.fingerprint),
      heading: self.ty.to_string(),
      identifier: self.identifier,
      metadata: &self.metadata,
    }
  }
}

impl Page for PackageTextHtml {
  fn class(&self) -> Option<&'static str> {
    Some("text")
  }

  fn script(&self) -> Option<&'static str> {
    Some("/static/item.js")
  }

  fn shortcuts(&self) -> Vec<Shortcut> {
    vec![
      Shortcut {
        description: "download",
        keys: &['d'],
      },
      Shortcut {
        description: "toggle full screen",
        keys: &['f'],
      },
    ]
  }

  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/item.css")
  }

  fn title(&self) -> String {
    format!("{} · Filepack", self.ty)
  }

  fn up(&self) -> Option<String> {
    Some(format!("/package/{}", self.identifier))
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn render() {
    let fingerprint = Fingerprint::EXAMPLE.parse::<Fingerprint>().unwrap();

    assert_eq!(
      PackageTextHtml {
        fingerprint,
        identifier: PackageIdentifier::Number(0),
        metadata: Metadata {
          creator: Some("foo".parse().unwrap()),
          title: Some("bar".parse().unwrap()),
          ..default()
        },
        text: "<baz>".into(),
        ty: PackageText::License(1),
      }
      .to_string(),
      unindent(&format!(
        "
          <pre>
          &lt;baz&gt;</pre>
          <footer>
            <hgroup>
              <h1>License 2</h1>
              <p>
                <span>foo</span>
                <a href=/package/0>
                  bar
                </a>
              </p>
            </hgroup>
            <a download href=/license/{fingerprint}/2>Download</a>
          </footer>

        "
      )),
    );
  }
}
