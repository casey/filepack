use super::*;

#[derive(Boilerplate)]
pub(crate) struct DocumentHtml {
  pub(crate) document: usize,
  pub(crate) fingerprint: Fingerprint,
  pub(crate) metadata: Metadata,
}

impl DocumentHtml {
  fn document(&self) -> &Item<Document> {
    let Media::Document { items } = self.metadata.media.as_ref().unwrap() else {
      unreachable!();
    };

    &items[self.document]
  }
}

impl Page for DocumentHtml {
  fn next(&self) -> Option<String> {
    self
      .metadata
      .media
      .as_ref()
      .unwrap()
      .next_item_url(self.fingerprint, self.document)
  }

  fn open_graph_image(&self) -> Option<OpenGraphImage> {
    OpenGraphImage::artwork(&self.metadata, self.fingerprint)
  }

  fn prev(&self) -> Option<String> {
    self
      .metadata
      .media
      .as_ref()
      .unwrap()
      .prev_item_url(self.fingerprint, self.document)
  }

  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/document.css")
  }

  fn title(&self) -> String {
    format!(
      "{} · filepack",
      self.document().display_title(self.document),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn document() {
    assert_eq!(
      DocumentHtml {
        document: 0,
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        metadata: Metadata {
          media: Some(Media::Document {
            items: vec![Item::test("foo.pdf")],
          }),
          ..default()
        },
      }
      .to_string(),
      format!(
        "<iframe src=/media/document/{}/item/1 title='Document 1'></iframe>\n",
        test::FINGERPRINT,
      ),
    );
  }

  #[test]
  fn navigation() {
    let html = DocumentHtml {
      document: 0,
      fingerprint: test::FINGERPRINT.parse().unwrap(),
      metadata: Metadata {
        media: Some(Media::Document {
          items: vec![Item::test("foo.pdf"), Item::test("bar.pdf")],
        }),
        ..default()
      },
    };

    assert_eq!(html.prev(), None);
    assert_eq!(
      html.next(),
      Some(format!("/package/{}/item/2", test::FINGERPRINT)),
    );
  }

  #[test]
  fn title() {
    let mut html = DocumentHtml {
      document: 0,
      fingerprint: test::FINGERPRINT.parse().unwrap(),
      metadata: Metadata {
        media: Some(Media::Document {
          items: vec![Item::test("foo.pdf")],
        }),
        ..default()
      },
    };

    assert_eq!(Page::title(&html), "Document 1 · filepack");

    let Some(Media::Document { items }) = html.metadata.media.as_mut() else {
      unreachable!();
    };

    items[0].title = Some("bar".parse().unwrap());

    assert_eq!(Page::title(&html), "bar · filepack");
  }
}
