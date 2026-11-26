use super::*;

#[derive(Boilerplate)]
#[cfg_attr(test, derive(Default))]
pub(crate) struct ItemHtml {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) identifier: PackageIdentifier,
  pub(crate) index: usize,
  pub(crate) metadata: Metadata,
  pub(crate) text: Option<String>,
}

impl ItemHtml {
  fn download_link(&self) -> String {
    let ty = match self.items() {
      Items::Audio(_) => "audio",
      Items::Image(_) => "image",
      Items::Text(_) => "text",
      Items::Video(_) => "video",
    };

    format!(
      "/media/{ty}/{}/item/{}",
      self.fingerprint,
      Ordinal(self.index),
    )
  }

  fn footer(&self) -> FooterHtml {
    FooterHtml {
      download: self.download_link(),
      heading: self.item().display_title(self.index),
      identifier: self.identifier,
      metadata: &self.metadata,
    }
  }

  fn item(&self) -> &dyn MediaItem {
    self.items().get(self.index).unwrap()
  }

  fn items(&self) -> Items {
    self.metadata.media.as_ref().unwrap().items().unwrap()
  }
}

impl Page for ItemHtml {
  fn class(&self) -> Option<&'static str> {
    if matches!(self.items(), Items::Text(_)) {
      Some("text")
    } else {
      None
    }
  }

  fn next(&self) -> Option<String> {
    self.items().next_url(self.identifier, self.index)
  }

  fn open_graph_image(&self) -> Option<OpenGraphImage> {
    match self.items() {
      Items::Audio(_) | Items::Text(_) => OpenGraphImage::artwork(&self.metadata, self.fingerprint),
      Items::Image(items) => Some(OpenGraphImage::thumbnail(
        &self.metadata,
        &items[self.index].content,
        format!(
          "media/image/{}/item/{}",
          self.fingerprint,
          Ordinal(self.index)
        ),
        self.item().display_title(self.index),
      )),
      Items::Video(_) => {
        if let Some(placeholder) = self.item().placeholder() {
          Some(OpenGraphImage::thumbnail(
            &self.metadata,
            placeholder,
            format!(
              "media/video/{}/item/{}/placeholder",
              self.fingerprint,
              Ordinal(self.index)
            ),
            format!("Placeholder for {}", self.item().display_title(self.index)),
          ))
        } else {
          OpenGraphImage::artwork(&self.metadata, self.fingerprint)
        }
      }
    }
  }

  fn open_graph_title(&self) -> Option<String> {
    Some(format!(
      "{} · {}",
      self.item().display_title(self.index),
      self.metadata.title.as_deref().unwrap_or(UNTITLED_PACKAGE),
    ))
  }

  fn prev(&self) -> Option<String> {
    self.items().prev_url(self.identifier, self.index)
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
    format!("{} · Filepack", self.item().display_title(self.index))
  }

  fn up(&self) -> Option<String> {
    Some(format!("/package/{}", self.identifier))
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn navigation() {
    for identifier in [
      PackageIdentifier::Fingerprint(Fingerprint::EXAMPLE.parse().unwrap()),
      PackageIdentifier::Number(0),
    ] {
      let mut html = ItemHtml {
        fingerprint: Fingerprint::EXAMPLE.parse().unwrap(),
        identifier,
        metadata: Metadata {
          media: Some(Media::Image {
            items: vec![Item::test("foo.png"), Item::test("bar.png")],
          }),
          ..default()
        },
        ..default()
      };

      assert_eq!(html.prev(), None);
      assert_eq!(html.next(), Some(format!("/package/{identifier}/item/2")));
      assert_eq!(html.up(), Some(format!("/package/{identifier}")));

      html.index = 1;

      assert_eq!(html.prev(), Some(format!("/package/{identifier}/item/1")));
      assert_eq!(html.next(), None);
    }
  }

  #[test]
  fn open_graph_image() {
    #[track_caller]
    fn case(metadata: Metadata, expected: Option<OpenGraphImage>) {
      assert_eq!(
        ItemHtml {
          fingerprint: Fingerprint::EXAMPLE.parse().unwrap(),
          identifier: PackageIdentifier::Fingerprint(Fingerprint::EXAMPLE.parse().unwrap()),
          metadata,
          ..default()
        }
        .open_graph_image(),
        expected,
      );
    }

    fn artwork() -> OpenGraphImage {
      OpenGraphImage {
        alt: "Artwork for Untitled package".into(),
        dimensions: Dimensions {
          height: 1,
          width: 1,
        },
        path: format!("artwork/{}", Fingerprint::EXAMPLE),
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
        alt: "Image 1".into(),
        dimensions: Dimensions {
          height: 2,
          width: 1,
        },
        path: format!("media/image/{}/item/1", Fingerprint::EXAMPLE),
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
        alt: "Image 1".into(),
        dimensions: Dimensions {
          height: 3,
          width: 4,
        },
        path: format!("media/image/{}/item/1/thumbnail", Fingerprint::EXAMPLE),
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
        alt: "Placeholder for Video 1".into(),
        dimensions: Dimensions {
          height: 2,
          width: 4,
        },
        path: format!("media/video/{}/item/1/placeholder", Fingerprint::EXAMPLE),
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
        alt: "Placeholder for Video 1".into(),
        dimensions: Dimensions {
          height: 1,
          width: 2,
        },
        path: format!(
          "media/video/{}/item/1/placeholder/thumbnail",
          Fingerprint::EXAMPLE
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
  fn open_graph_title() {
    assert_eq!(
      ItemHtml {
        fingerprint: Fingerprint::EXAMPLE.parse().unwrap(),
        identifier: PackageIdentifier::Fingerprint(Fingerprint::EXAMPLE.parse().unwrap()),
        metadata: Metadata {
          media: Some(Media::Image {
            items: vec![Item::test("foo.png")],
          }),
          ..default()
        },
        ..default()
      }
      .open_graph_title(),
      Some("Image 1 · Untitled package".into()),
    );
  }

  #[test]
  fn render() {
    #[track_caller]
    fn case(metadata: Metadata, text: Option<&str>, expected: String) {
      assert_eq!(
        ItemHtml {
          fingerprint: Fingerprint::EXAMPLE.parse().unwrap(),
          identifier: PackageIdentifier::Fingerprint(Fingerprint::EXAMPLE.parse().unwrap()),
          metadata,
          text: text.map(str::to_owned),
          ..default()
        }
        .to_string(),
        unindent(&expected),
      );
    }

    let fingerprint = Fingerprint::EXAMPLE;

    case(
      Metadata {
        creator: Some("baz".parse().unwrap()),
        media: Some(Media::Audio {
          items: vec![Item::test("foo.flac")],
        }),
        title: Some("qux".parse().unwrap()),
        ..default()
      },
      None,
      format!(
        "
          <img src=/artwork/{fingerprint}>
          <audio autofocus controls src=/media/audio/{fingerprint}/item/1></audio>
          <footer>
            <hgroup>
              <h1>Track 1</h1>
              <p>
                <span>baz</span>
                <a href=/package/{fingerprint}>
                  qux
                </a>
              </p>
            </hgroup>
            <a download href=/media/audio/{fingerprint}/item/1>Download</a>
          </footer>

        "
      ),
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
      None,
      format!(
        "
          <img src=/media/image/{fingerprint}/item/1 width=1 height=2>
          <footer>
            <hgroup>
              <h1>Image 1</h1>
              <p>
                <a href=/package/{fingerprint}>
                  <code>{fingerprint}</code>
                </a>
              </p>
            </hgroup>
            <a download href=/media/image/{fingerprint}/item/1>Download</a>
          </footer>

        "
      ),
    );

    case(
      Metadata {
        language: Some("en".parse().unwrap()),
        media: Some(Media::Text {
          items: vec![Item {
            content: Text::test("foo.md"),
            title: Some("bar".parse().unwrap()),
          }],
        }),
        ..default()
      },
      Some("\n<baz>"),
      format!(
        "
          <pre lang='en'>

          &lt;baz&gt;</pre>
          <footer>
            <hgroup>
              <h1>bar</h1>
              <p>
                <a href=/package/{fingerprint}>
                  <code>{fingerprint}</code>
                </a>
              </p>
            </hgroup>
            <a download href=/media/text/{fingerprint}/item/1>Download</a>
          </footer>

        "
      ),
    );

    case(
      Metadata {
        media: Some(Media::Video {
          items: vec![Item::test("foo.mp4")],
        }),
        ..default()
      },
      None,
      format!(
        "
          <video
            autofocus
            controls
            src=/media/video/{fingerprint}/item/1></video>
          <footer>
            <hgroup>
              <h1>Video 1</h1>
              <p>
                <a href=/package/{fingerprint}>
                  <code>{fingerprint}</code>
                </a>
              </p>
            </hgroup>
            <a download href=/media/video/{fingerprint}/item/1>Download</a>
          </footer>

        "
      ),
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
      None,
      format!(
        "
          <video
            autofocus
            controls
            poster=/media/video/{fingerprint}/item/1/placeholder
            src=/media/video/{fingerprint}/item/1></video>
          <footer>
            <hgroup>
              <h1>Video 1</h1>
              <p>
                <a href=/package/{fingerprint}>
                  <code>{fingerprint}</code>
                </a>
              </p>
            </hgroup>
            <a download href=/media/video/{fingerprint}/item/1>Download</a>
          </footer>

        "
      ),
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
      None,
      format!(
        "
          <img src=/media/image/{fingerprint}/item/1 width=1 height=1>
          <footer>
            <hgroup>
              <h1>Image 1</h1>
              <p>
                <span>bar</span>
                <a href=/package/{fingerprint}>
                  baz
                </a>
              </p>
            </hgroup>
            <a download href=/media/image/{fingerprint}/item/1>Download</a>
          </footer>

        "
      ),
    );

    assert_eq!(
      ItemHtml {
        fingerprint: Fingerprint::EXAMPLE.parse().unwrap(),
        identifier: PackageIdentifier::Number(0),
        metadata: Metadata {
          media: Some(Media::Image {
            items: vec![Item::test("foo/bar baz.png")],
          }),
          ..default()
        },
        ..default()
      }
      .to_string(),
      unindent(&format!(
        "
          <img src=/media/image/{fingerprint}/item/1 width=1 height=1>
          <footer>
            <hgroup>
              <h1>Image 1</h1>
              <p>
                <a href=/package/0>
                  0
                </a>
              </p>
            </hgroup>
            <a download href=/media/image/{fingerprint}/item/1>Download</a>
          </footer>

        "
      )),
    );
  }

  #[test]
  fn shortcuts() {
    assert_eq!(
      ItemHtml::default().shortcuts(),
      vec![
        Shortcut {
          description: "download",
          keys: &['d'],
        },
        Shortcut {
          description: "toggle full screen",
          keys: &['f'],
        },
      ],
    );
  }

  #[test]
  fn title() {
    #[track_caller]
    fn case(media: Media, expected: &str) {
      assert_eq!(
        Page::title(&ItemHtml {
          fingerprint: Fingerprint::EXAMPLE.parse().unwrap(),
          identifier: PackageIdentifier::Fingerprint(Fingerprint::EXAMPLE.parse().unwrap()),
          metadata: Metadata {
            media: Some(media),
            ..default()
          },
          ..default()
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
      Media::Text {
        items: vec![Item::test("foo.md")],
      },
      "Text 1 · Filepack",
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
