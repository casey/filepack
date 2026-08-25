use super::*;

#[derive(Boilerplate)]
pub(crate) struct VideoHtml {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) metadata: Metadata,
  pub(crate) video: usize,
}

impl VideoHtml {
  fn video(&self) -> &Item<Video> {
    let Media::Video { items } = self.metadata.media.as_ref().unwrap() else {
      unreachable!();
    };

    &items[self.video]
  }
}

impl Page for VideoHtml {
  fn open_graph_image(&self) -> Option<OpenGraphImage> {
    OpenGraphImage::artwork(&self.metadata, self.fingerprint)
  }

  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/video.css")
  }

  fn title(&self) -> String {
    format!("{} · filepack", self.video().display_title())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn open_graph_image() {
    let html = VideoHtml {
      fingerprint: test::FINGERPRINT.parse().unwrap(),
      metadata: Metadata {
        artwork: Some("foo.png".parse().unwrap()),
        ..default()
      },
      video: 0,
    };

    assert_eq!(
      html.open_graph_image(),
      Some(OpenGraphImage {
        dimensions: Dimensions::default(),
        path: format!("artwork/{}", test::FINGERPRINT),
      }),
    );

    let html = VideoHtml {
      fingerprint: test::FINGERPRINT.parse().unwrap(),
      metadata: default(),
      video: 0,
    };

    assert_eq!(html.open_graph_image(), None);
  }

  #[test]
  fn title() {
    let html = VideoHtml {
      fingerprint: test::FINGERPRINT.parse().unwrap(),
      metadata: Metadata {
        media: Some(Media::Video {
          items: vec![Item {
            content: "foo.mp4".parse().unwrap(),
            title: Some("bar".parse().unwrap()),
          }],
        }),
        ..default()
      },
      video: 0,
    };

    assert_eq!(Page::title(&html), "bar · filepack");
  }
}
