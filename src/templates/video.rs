use super::*;

#[derive(Boilerplate)]
pub(crate) struct VideoHtml {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) metadata: Metadata,
  pub(crate) video: usize,
}

impl Page for VideoHtml {
  fn open_graph_image(&self) -> Option<OpenGraphImage> {
    OpenGraphImage::artwork(&self.metadata, self.fingerprint)
  }

  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/video.css")
  }

  fn title(&self) -> String {
    format!("video {} · filepack", self.video)
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
}
