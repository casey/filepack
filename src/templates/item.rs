use super::*;

#[derive(Boilerplate)]
pub(crate) struct ItemHtml {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) index: usize,
  pub(crate) metadata: Metadata,
}

impl ItemHtml {
  fn item(&self) -> &dyn MediaItem {
    self.media().item(self.index).unwrap()
  }

  fn media(&self) -> &Media {
    self.metadata.media.as_ref().unwrap()
  }
}

impl Page for ItemHtml {
  fn next(&self) -> Option<String> {
    self.media().next_item_url(self.fingerprint, self.index)
  }

  fn open_graph_image(&self) -> Option<OpenGraphImage> {
    match self.media() {
      Media::Audio { .. } => OpenGraphImage::artwork(&self.metadata, self.fingerprint),
      Media::Image { items } => Some(OpenGraphImage::thumbnail(
        &self.metadata,
        &items[self.index].content,
        format!(
          "media/image/{}/item/{}",
          self.fingerprint,
          Ordinal(self.index)
        ),
      )),
      Media::Video { .. } => {
        if let Some(placeholder) = self.item().placeholder() {
          Some(OpenGraphImage::thumbnail(
            &self.metadata,
            placeholder,
            format!(
              "media/video/{}/item/{}/placeholder",
              self.fingerprint,
              Ordinal(self.index)
            ),
          ))
        } else {
          OpenGraphImage::artwork(&self.metadata, self.fingerprint)
        }
      }
      Media::Web => unreachable!(),
    }
  }

  fn prev(&self) -> Option<String> {
    self.media().prev_item_url(self.fingerprint, self.index)
  }

  fn script(&self) -> Option<&'static str> {
    Some("/static/item.js")
  }

  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/item.css")
  }

  fn title(&self) -> String {
    format!("{} · Filepack", self.item().display_title(self.index))
  }

  fn up(&self) -> Option<String> {
    Some(format!("/package/{}", self.fingerprint))
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn navigation() {
    let mut html = ItemHtml {
      fingerprint: test::FINGERPRINT.parse().unwrap(),
      index: 0,
      metadata: Metadata {
        media: Some(Media::Image {
          items: vec![Item::test("foo.png"), Item::test("bar.png")],
        }),
        ..default()
      },
    };

    assert_eq!(html.prev(), None);
    assert_eq!(
      html.next(),
      Some(format!("/package/{}/item/2", test::FINGERPRINT)),
    );
    assert_eq!(html.up(), Some(format!("/package/{}", test::FINGERPRINT)));

    html.index = 1;

    assert_eq!(
      html.prev(),
      Some(format!("/package/{}/item/1", test::FINGERPRINT)),
    );
    assert_eq!(html.next(), None);
  }

  #[test]
  fn open_graph_image() {
    #[track_caller]
    fn case(metadata: Metadata, expected: Option<OpenGraphImage>) {
      assert_eq!(
        ItemHtml {
          fingerprint: test::FINGERPRINT.parse().unwrap(),
          index: 0,
          metadata,
        }
        .open_graph_image(),
        expected,
      );
    }

    fn artwork() -> OpenGraphImage {
      OpenGraphImage {
        dimensions: Dimensions {
          height: 1,
          width: 1,
        },
        path: format!("artwork/{}", test::FINGERPRINT),
      }
    }

    case(
      Metadata {
        artwork: Some(Image::test("foo.png")),
        media: Some(Media::Audio {
          items: vec![Item::test("foo.flac")],
        }),
        ..default()
      },
      Some(artwork()),
    );

    case(
      Metadata {
        media: Some(Media::Audio {
          items: vec![Item::test("foo.flac")],
        }),
        ..default()
      },
      None,
    );

    let image = Image {
      dimensions: Dimensions {
        height: 1,
        width: 2,
      },
      orientation: Orientation {
        mirrored: false,
        rotation: Rotation::R90,
      },
      ..Image::test("foo.png")
    };

    case(
      Metadata {
        media: Some(Media::Image {
          items: vec![Item {
            content: image.clone(),
            title: None,
          }],
        }),
        ..default()
      },
      Some(OpenGraphImage {
        dimensions: Dimensions {
          height: 2,
          width: 1,
        },
        path: format!("media/image/{}/item/1", test::FINGERPRINT),
      }),
    );

    case(
      Metadata {
        media: Some(Media::Image {
          items: vec![Item {
            content: image,
            title: None,
          }],
        }),
        thumbnails: Some(
          [(
            "foo.png".parse().unwrap(),
            Image {
              dimensions: Dimensions {
                height: 3,
                width: 4,
              },
              ..Image::test("thumbnails/foo.jpg")
            },
          )]
          .into(),
        ),
        ..default()
      },
      Some(OpenGraphImage {
        dimensions: Dimensions {
          height: 3,
          width: 4,
        },
        path: format!("media/image/{}/item/1/thumbnail", test::FINGERPRINT),
      }),
    );

    case(
      Metadata {
        artwork: Some(Image::test("foo.png")),
        media: Some(Media::Video {
          items: vec![Item {
            content: Video {
              placeholder: Some(Image {
                dimensions: Dimensions {
                  height: 2,
                  width: 4,
                },
                ..Image::test("bar.png")
              }),
              ..Video::test("foo.mp4")
            },
            title: None,
          }],
        }),
        ..default()
      },
      Some(OpenGraphImage {
        dimensions: Dimensions {
          height: 2,
          width: 4,
        },
        path: format!("media/video/{}/item/1/placeholder", test::FINGERPRINT),
      }),
    );

    case(
      Metadata {
        media: Some(Media::Video {
          items: vec![Item {
            content: Video {
              placeholder: Some(Image::test("bar.png")),
              ..Video::test("foo.mp4")
            },
            title: None,
          }],
        }),
        thumbnails: Some(
          [(
            "bar.png".parse().unwrap(),
            Image {
              dimensions: Dimensions {
                height: 1,
                width: 2,
              },
              ..Image::test("thumbnails/bar.jpg")
            },
          )]
          .into(),
        ),
        ..default()
      },
      Some(OpenGraphImage {
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        path: format!(
          "media/video/{}/item/1/placeholder/thumbnail",
          test::FINGERPRINT
        ),
      }),
    );

    case(
      Metadata {
        artwork: Some(Image::test("foo.png")),
        media: Some(Media::Video {
          items: vec![Item::test("foo.mp4")],
        }),
        ..default()
      },
      Some(artwork()),
    );

    case(
      Metadata {
        media: Some(Media::Video {
          items: vec![Item::test("foo.mp4")],
        }),
        ..default()
      },
      None,
    );
  }

  #[test]
  fn render() {
    #[track_caller]
    fn case(metadata: Metadata, expected: &str) {
      assert_eq!(
        ItemHtml {
          fingerprint: test::FINGERPRINT.parse().unwrap(),
          index: 0,
          metadata,
        }
        .to_string(),
        unindent(&expected.replace("{fingerprint}", test::FINGERPRINT)),
      );
    }

    case(
      Metadata {
        media: Some(Media::Audio {
          items: vec![Item::test("foo.flac")],
        }),
        ..default()
      },
      "
        <img src=/artwork/{fingerprint}>
        <hgroup>
          <h1>Track 1</h1>
          <p>
            <a href=/package/{fingerprint}>
              <code>{fingerprint}</code>
            </a>
          </p>
        </hgroup>
        <audio controls src=/media/audio/{fingerprint}/item/1></audio>
      ",
    );

    case(
      Metadata {
        media: Some(Media::Image {
          items: vec![Item {
            content: Image {
              dimensions: Dimensions {
                height: 1,
                width: 2,
              },
              orientation: Orientation {
                mirrored: false,
                rotation: Rotation::R90,
              },
              ..Image::test("foo.png")
            },
            title: None,
          }],
        }),
        ..default()
      },
      "
        <img src=/media/image/{fingerprint}/item/1 width=1 height=2>
        <hgroup>
          <h1>Image 1</h1>
          <p>
            <a href=/package/{fingerprint}>
              <code>{fingerprint}</code>
            </a>
          </p>
        </hgroup>
      ",
    );

    case(
      Metadata {
        media: Some(Media::Video {
          items: vec![Item::test("foo.mp4")],
        }),
        ..default()
      },
      "
        <video
          controls
          src=/media/video/{fingerprint}/item/1></video>
        <hgroup>
          <h1>Video 1</h1>
          <p>
            <a href=/package/{fingerprint}>
              <code>{fingerprint}</code>
            </a>
          </p>
        </hgroup>
      ",
    );

    case(
      Metadata {
        media: Some(Media::Video {
          items: vec![Item {
            content: Video {
              placeholder: Some(Image::test("bar.png")),
              ..Video::test("foo.mp4")
            },
            title: None,
          }],
        }),
        ..default()
      },
      "
        <video
          controls
          poster=/media/video/{fingerprint}/item/1/placeholder
          src=/media/video/{fingerprint}/item/1></video>
        <hgroup>
          <h1>Video 1</h1>
          <p>
            <a href=/package/{fingerprint}>
              <code>{fingerprint}</code>
            </a>
          </p>
        </hgroup>
      ",
    );

    case(
      Metadata {
        creator: Some("bar".parse().unwrap()),
        media: Some(Media::Image {
          items: vec![Item::test("foo.png")],
        }),
        title: Some("baz".parse().unwrap()),
        ..default()
      },
      "
        <img src=/media/image/{fingerprint}/item/1 width=1 height=1>
        <hgroup>
          <h1>Image 1</h1>
          <p>bar</p>
          <p>
            <a href=/package/{fingerprint}>
              baz
            </a>
          </p>
        </hgroup>
      ",
    );
  }

  #[test]
  fn title() {
    #[track_caller]
    fn case(media: Media, expected: &str) {
      assert_eq!(
        Page::title(&ItemHtml {
          fingerprint: test::FINGERPRINT.parse().unwrap(),
          index: 0,
          metadata: Metadata {
            media: Some(media),
            ..default()
          },
        }),
        expected,
      );
    }

    case(
      Media::Audio {
        items: vec![Item::test("foo.flac")],
      },
      "Track 1 · Filepack",
    );

    case(
      Media::Image {
        items: vec![Item::test("foo.png")],
      },
      "Image 1 · Filepack",
    );

    case(
      Media::Video {
        items: vec![Item::test("foo.mp4")],
      },
      "Video 1 · Filepack",
    );

    case(
      Media::Image {
        items: vec![Item {
          title: Some("bar".parse().unwrap()),
          ..Item::test("foo.png")
        }],
      },
      "bar · Filepack",
    );
  }
}
