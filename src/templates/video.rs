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
    format!("{} · filepack", self.video().display_title(self.video))
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
        artwork: Some(Image::test("foo.png")),
        ..default()
      },
      video: 0,
    };

    assert_eq!(
      html.open_graph_image(),
      Some(OpenGraphImage {
        dimensions: Dimensions {
          height: 1,
          width: 1
        },
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
  fn render() {
    #[track_caller]
    fn case(placeholder: Option<Image>, expected: String) {
      let html = VideoHtml {
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        metadata: Metadata {
          media: Some(Media::Video {
            items: vec![Item {
              content: Video {
                placeholder,
                ..Video::test("foo.mp4")
              },
              title: None,
            }],
          }),
          ..default()
        },
        video: 0,
      };

      assert_eq!(html.to_string(), expected);
    }

    case(
      None,
      format!(
        "<video\n  controls\n  src=/media/video/{}/item/1></video>\n",
        test::FINGERPRINT,
      ),
    );

    case(
      Some(Image::test("bar.png")),
      format!(
        "<video\n  controls\n  poster=/media/video/{0}/item/1/placeholder\n  src=/media/video/{0}/item/1></video>\n",
        test::FINGERPRINT,
      ),
    );
  }

  #[test]
  fn title() {
    let mut html = VideoHtml {
      fingerprint: test::FINGERPRINT.parse().unwrap(),
      metadata: Metadata {
        media: Some(Media::Video {
          items: vec![Item::test("foo.mp4")],
        }),
        ..default()
      },
      video: 0,
    };

    assert_eq!(Page::title(&html), "Video 1 · filepack");

    let Some(Media::Video { items }) = html.metadata.media.as_mut() else {
      unreachable!();
    };

    items[0].title = Some("bar".parse().unwrap());

    assert_eq!(Page::title(&html), "bar · filepack");
  }
}
