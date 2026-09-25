use super::*;

#[derive(Boilerplate)]
pub struct PackageHtml {
  pub colophon: Option<Hash>,
  pub directory: Directory,
  pub fingerprint: Fingerprint,
  pub identifier: PackageIdentifier,
  pub metadata: Option<Metadata>,
  pub mounted: bool,
  pub next: Option<u64>,
  pub number: Option<u64>,
  pub prev: Option<u64>,
  pub readme: Option<Hash>,
  pub revision: Option<Revision>,
  pub totals: Totals,
}

impl PackageHtml {
  fn info(&self) -> Info {
    InfoBuilder::new()
      .when_some(self.number, |builder, number| {
        builder.link("number", number, format!("/package/{number}"))
      })
      .when_some(self.revision, |builder, revision| {
        builder.code_link("revision", revision, format!("/package/{revision}"))
      })
      .code_link(
        "fingerprint",
        self.fingerprint,
        format!("/package/{}", self.fingerprint),
      )
      .value("size", format_size(self.totals.file_size))
      .link(
        "files",
        Count::new(self.totals.files, "file").to_string(),
        format!("/directory/{}", Hash::from(self.fingerprint)),
      )
      .when(self.mounted, |builder| {
        builder.link("mount", "view", format!("/mount/{}/", self.fingerprint))
      })
      .when_some(self.metadata.as_ref(), |builder, metadata| {
        metadata.info(builder, self.identifier, self.readme, self.colophon)
      })
      .build()
  }

  fn title(&self) -> Option<&str> {
    self.metadata.as_ref()?.title.as_deref()
  }

  fn videos(
    &self,
  ) -> impl Iterator<Item = (&Item<Video>, Duration, Option<&Image>, Option<Dimensions>)> {
    let metadata = self.metadata.as_ref().unwrap();

    let Media::Video { items } = metadata.media.as_ref().unwrap() else {
      unreachable!();
    };

    items.iter().map(|video| {
      let duration = Duration::from_millis(video.content.duration);

      let image = video
        .content
        .placeholder
        .as_ref()
        .map(|placeholder| metadata.thumbnail(&placeholder.path).unwrap_or(placeholder));

      let dimensions = image
        .map(Image::oriented_dimensions)
        .or_else(|| video.content.oriented_dimensions());

      (video, duration, image, dimensions)
    })
  }
}

impl Page for PackageHtml {
  fn next(&self) -> Option<String> {
    self.next.map(|number| format!("/package/{number}"))
  }

  fn open_graph_description(&self) -> Option<String> {
    Some(self.metadata.as_ref()?.description.as_ref()?.to_string())
  }

  fn open_graph_image(&self) -> Option<OpenGraphImage> {
    OpenGraphImage::artwork(self.metadata.as_ref()?, self.fingerprint)
  }

  fn prev(&self) -> Option<String> {
    self.prev.map(|number| format!("/package/{number}"))
  }

  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/package.css")
  }

  fn title(&self) -> String {
    if let Some(title) = self.title() {
      format!("{title} · Filepack")
    } else {
      format!("{} · Filepack", self.fingerprint)
    }
  }

  fn up(&self) -> Option<String> {
    Some("/".into())
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn audio() {
    let metadata = Metadata {
      media: Some(Media::Audio {
        items: vec![
          Item {
            content: Audio {
              channels: 2,
              path: "foo.flac".parse().unwrap(),
              sample_bits: Some(16),
              sample_rate: 44100,
              samples: 9_922_500,
              size: 0,
              ty: Some(AudioType::Flac),
            },
            title: Some("foo".parse().unwrap()),
          },
          Item {
            content: Audio {
              channels: 2,
              path: "bar.flac".parse().unwrap(),
              sample_bits: Some(24),
              sample_rate: 96000,
              samples: 96000,
              size: 0,
              ty: Some(AudioType::Flac),
            },
            title: None,
          },
        ],
      }),
      ..default()
    };

    assert_eq!(
      PackageHtml {
        colophon: None,
        directory: Directory::new(),
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        identifier: PackageIdentifier::Fingerprint(test::FINGERPRINT.parse().unwrap()),
        metadata: Some(metadata),
        mounted: false,
        next: None,
        number: Some(1),
        prev: None,
        readme: None,
        revision: None,
        totals: Totals {
          directories: 0,
          directory_size: 0,
          file_size: 6,
          files: 2,
        },
      }
      .to_string(),
      unindent(&format!(
        "
          <dl>
            <div>
              <dt>number</dt>
              <dd>
                <a href='/package/1'>1</a>
              </dd>
            </div>
            <div>
              <dt>fingerprint</dt>
              <dd>
                <a href='/package/{fingerprint}'><code>{fingerprint}</code></a>
              </dd>
            </div>
            <div>
              <dt>size</dt>
              <dd>
                6 B
              </dd>
            </div>
            <div>
              <dt>files</dt>
              <dd>
                <a href='/directory/{hash}'>2 files</a>
              </dd>
            </div>
            <div>
              <dt>media</dt>
              <dd>
                <a href='/package/{fingerprint}/media'>audio</a>
              </dd>
            </div>
            <div>
              <dt>tracks</dt>
              <dd>
                2
              </dd>
            </div>
            <div>
              <dt>duration</dt>
              <dd>
                3:46
              </dd>
            </div>
            <div>
              <dt>format</dt>
              <dd>
                <ol role=list>
                  <li>
                    FLAC
                  </li>
                </ol>
              </dd>
            </div>
          </dl>
          <ol>
            <li>
              <a href=/package/{fingerprint}/item/1>foo</a>
              <time datetime=PT3M45S>3:45</time>
            </li>
            <li>
              <a href=/package/{fingerprint}/item/2>Track 2</a>
              <time datetime=PT0M1S>0:01</time>
            </li>
          </ol>
        ",
        fingerprint = test::FINGERPRINT,
        hash = test::HASH,
      )),
    );
  }

  #[test]
  fn duration_saturates() {
    let audio = Item {
      content: Audio {
        channels: 2,
        path: "foo.flac".parse().unwrap(),
        sample_bits: Some(16),
        sample_rate: 1,
        samples: u64::MAX,
        size: 0,
        ty: Some(AudioType::Flac),
      },
      title: Some("foo".parse().unwrap()),
    };

    let metadata = Metadata {
      media: Some(Media::Audio {
        items: vec![audio.clone(), audio],
      }),
      ..default()
    };

    assert_eq!(
      PackageHtml {
        colophon: None,
        directory: Directory::new(),
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        identifier: PackageIdentifier::Fingerprint(test::FINGERPRINT.parse().unwrap()),
        metadata: Some(metadata),
        mounted: false,
        next: None,
        number: Some(1),
        prev: None,
        readme: None,
        revision: None,
        totals: Totals {
          directories: 0,
          directory_size: 0,
          file_size: 6,
          files: 2,
        },
      }
      .to_string(),
      unindent(&format!(
        "
          <dl>
            <div>
              <dt>number</dt>
              <dd>
                <a href='/package/1'>1</a>
              </dd>
            </div>
            <div>
              <dt>fingerprint</dt>
              <dd>
                <a href='/package/{fingerprint}'><code>{fingerprint}</code></a>
              </dd>
            </div>
            <div>
              <dt>size</dt>
              <dd>
                6 B
              </dd>
            </div>
            <div>
              <dt>files</dt>
              <dd>
                <a href='/directory/{hash}'>2 files</a>
              </dd>
            </div>
            <div>
              <dt>media</dt>
              <dd>
                <a href='/package/{fingerprint}/media'>audio</a>
              </dd>
            </div>
            <div>
              <dt>tracks</dt>
              <dd>
                2
              </dd>
            </div>
            <div>
              <dt>duration</dt>
              <dd>
                5124095576030431:00:15
              </dd>
            </div>
            <div>
              <dt>format</dt>
              <dd>
                <ol role=list>
                  <li>
                    FLAC
                  </li>
                </ol>
              </dd>
            </div>
          </dl>
          <ol>
            <li>
              <a href=/package/{fingerprint}/item/1>foo</a>
              <time datetime=PT5124095576030431H0M15S>5124095576030431:00:15</time>
            </li>
            <li>
              <a href=/package/{fingerprint}/item/2>foo</a>
              <time datetime=PT5124095576030431H0M15S>5124095576030431:00:15</time>
            </li>
          </ol>
        ",
        fingerprint = test::FINGERPRINT,
        hash = test::HASH,
      )),
    );
  }

  #[test]
  fn image() {
    let metadata = Metadata {
      media: Some(Media::Image {
        items: vec![
          Item {
            content: Image {
              alpha: false,
              bit_depth: 8,
              chroma_subsampling: None,
              color_type: Some(ColorType::Rgb),
              dimensions: Dimensions {
                height: 1,
                width: 2,
              },
              orientation: Orientation::new(),
              path: "foo.png".parse().unwrap(),
              ty: Some(ImageType::Png),
            },
            title: None,
          },
          Item {
            content: Image {
              alpha: false,
              bit_depth: 8,
              chroma_subsampling: None,
              color_type: Some(ColorType::Rgb),
              dimensions: Dimensions::default(),
              orientation: Orientation::new(),
              path: "bar.jpg".parse().unwrap(),
              ty: Some(ImageType::Jpeg),
            },
            title: Some("quux".parse().unwrap()),
          },
          Item {
            content: Image {
              alpha: false,
              bit_depth: 8,
              chroma_subsampling: None,
              color_type: Some(ColorType::Rgb),
              dimensions: Dimensions {
                height: 1,
                width: 2,
              },
              orientation: Orientation::new(),
              path: "baz.png".parse().unwrap(),
              ty: Some(ImageType::Png),
            },
            title: None,
          },
        ],
      }),
      publisher: Some("qux".parse().unwrap()),
      ..default()
    };

    assert_eq!(
      PackageHtml {
        colophon: None,
        directory: Directory::new(),
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        identifier: PackageIdentifier::Fingerprint(test::FINGERPRINT.parse().unwrap()),
        metadata: Some(metadata),
        mounted: false,
        next: None,
        number: Some(1),
        prev: None,
        readme: None,
        revision: None,
        totals: Totals {
          directories: 0,
          directory_size: 0,
          file_size: 9,
          files: 3,
        },
      }
      .to_string(),
      unindent(&format!(
        r#"
          <dl>
            <div>
              <dt>number</dt>
              <dd>
                <a href='/package/1'>1</a>
              </dd>
            </div>
            <div>
              <dt>fingerprint</dt>
              <dd>
                <a href='/package/{fingerprint}'><code>{fingerprint}</code></a>
              </dd>
            </div>
            <div>
              <dt>size</dt>
              <dd>
                9 B
              </dd>
            </div>
            <div>
              <dt>files</dt>
              <dd>
                <a href='/directory/{hash}'>3 files</a>
              </dd>
            </div>
            <div>
              <dt>publisher</dt>
              <dd>
                qux
              </dd>
            </div>
            <div>
              <dt>media</dt>
              <dd>
                <a href='/package/{fingerprint}/media'>image</a>
              </dd>
            </div>
            <div>
              <dt>images</dt>
              <dd>
                3
              </dd>
            </div>
            <div>
              <dt>format</dt>
              <dd>
                <ol role=list>
                  <li>
                    PNG
                  </li>
                  <li>
                    JPEG
                  </li>
                </ol>
              </dd>
            </div>
          </dl>
          <ul class=thumbnails>
            <li style="--aspect-ratio: 2 / 1">
              <a href=/package/{fingerprint}/item/1>
                <figure>
                  <img loading=lazy src=/media/image/{fingerprint}/item/1/thumbnail>
                </figure>
              </a>
            </li>
            <li style="--aspect-ratio: 1 / 1">
              <a href=/package/{fingerprint}/item/2>
                <figure>
                  <img loading=lazy src=/media/image/{fingerprint}/item/2/thumbnail>
                  <figcaption>quux</figcaption>
                </figure>
              </a>
            </li>
            <li style="--aspect-ratio: 2 / 1">
              <a href=/package/{fingerprint}/item/3>
                <figure>
                  <img loading=lazy src=/media/image/{fingerprint}/item/3/thumbnail>
                </figure>
              </a>
            </li>
          </ul>
        "#,
        fingerprint = test::FINGERPRINT,
        hash = test::HASH,
      )),
    );
  }

  #[test]
  fn links_by_number() {
    let metadata = Metadata {
      media: Some(Media::Audio {
        items: vec![Item {
          content: Audio {
            channels: 2,
            path: "foo.flac".parse().unwrap(),
            sample_bits: Some(16),
            sample_rate: 44100,
            samples: 9_922_500,
            size: 0,
            ty: Some(AudioType::Flac),
          },
          title: Some("foo".parse().unwrap()),
        }],
      }),
      ..default()
    };

    assert_eq!(
      PackageHtml {
        colophon: None,
        directory: Directory::new(),
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        identifier: PackageIdentifier::Number(1),
        metadata: Some(metadata),
        mounted: false,
        next: None,
        number: Some(1),
        prev: None,
        readme: None,
        revision: Some(test::REVISION.parse().unwrap()),
        totals: Totals {
          directories: 0,
          directory_size: 0,
          file_size: 3,
          files: 1,
        },
      }
      .to_string(),
      unindent(&format!(
        "
          <dl>
            <div>
              <dt>number</dt>
              <dd>
                <a href='/package/1'>1</a>
              </dd>
            </div>
            <div>
              <dt>revision</dt>
              <dd>
                <a href='/package/{revision}'><code>{revision}</code></a>
              </dd>
            </div>
            <div>
              <dt>fingerprint</dt>
              <dd>
                <a href='/package/{fingerprint}'><code>{fingerprint}</code></a>
              </dd>
            </div>
            <div>
              <dt>size</dt>
              <dd>
                3 B
              </dd>
            </div>
            <div>
              <dt>files</dt>
              <dd>
                <a href='/directory/{hash}'>1 file</a>
              </dd>
            </div>
            <div>
              <dt>media</dt>
              <dd>
                <a href='/package/1/media'>audio</a>
              </dd>
            </div>
            <div>
              <dt>tracks</dt>
              <dd>
                1
              </dd>
            </div>
            <div>
              <dt>duration</dt>
              <dd>
                3:45
              </dd>
            </div>
            <div>
              <dt>format</dt>
              <dd>
                <ol role=list>
                  <li>
                    FLAC
                  </li>
                </ol>
              </dd>
            </div>
          </dl>
          <ol>
            <li>
              <a href=/package/1/item/1>foo</a>
              <time datetime=PT3M45S>3:45</time>
            </li>
          </ol>
        ",
        fingerprint = test::FINGERPRINT,
        revision = test::REVISION,
        hash = test::HASH,
      )),
    );
  }

  #[test]
  fn navigation() {
    let mut html = PackageHtml {
      colophon: None,
      directory: Directory::new(),
      fingerprint: test::FINGERPRINT.parse().unwrap(),
      identifier: PackageIdentifier::Number(2),
      metadata: None,
      mounted: false,
      next: Some(3),
      number: Some(2),
      prev: Some(1),
      readme: None,
      revision: None,
      totals: Totals::default(),
    };

    assert_eq!(html.next(), Some("/package/3".into()));
    assert_eq!(html.prev(), Some("/package/1".into()));
    assert_eq!(html.up(), Some("/".into()));

    html.next = None;
    html.prev = None;

    assert_eq!(html.next(), None);
    assert_eq!(html.prev(), None);
  }

  #[test]
  fn open_graph_metadata() {
    let html = PackageHtml {
      colophon: None,
      directory: Directory::new(),
      fingerprint: test::FINGERPRINT.parse().unwrap(),
      identifier: PackageIdentifier::Fingerprint(test::FINGERPRINT.parse().unwrap()),
      metadata: Some(Metadata {
        artwork: Some(Image {
          alpha: false,
          bit_depth: 8,
          chroma_subsampling: None,
          color_type: Some(ColorType::Rgb),
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
          orientation: Orientation {
            mirrored: false,
            rotation: Rotation::R90,
          },
          path: "foo.png".parse().unwrap(),
          ty: Some(ImageType::Png),
        }),
        description: Some("bar".parse().unwrap()),
        ..default()
      }),
      mounted: false,
      next: None,
      number: Some(1),
      prev: None,
      readme: None,
      revision: None,
      totals: Totals::default(),
    };

    assert_eq!(
      html.open_graph_image(),
      Some(OpenGraphImage {
        dimensions: Dimensions {
          height: 2,
          width: 1,
        },
        path: format!("artwork/{}", test::FINGERPRINT),
      }),
    );

    assert_eq!(html.open_graph_description(), Some("bar".into()));

    let mut html = html;

    html.metadata.as_mut().unwrap().thumbnails = Some(
      [(
        "foo.png".parse().unwrap(),
        Image {
          alpha: false,
          bit_depth: 8,
          chroma_subsampling: None,
          color_type: Some(ColorType::Rgb),
          dimensions: Dimensions {
            height: 3,
            width: 4,
          },
          orientation: Orientation::default(),
          path: "thumbnails/foo.jpg".parse().unwrap(),
          ty: Some(ImageType::Jpeg),
        },
      )]
      .into(),
    );

    assert_eq!(
      html.open_graph_image(),
      Some(OpenGraphImage {
        dimensions: Dimensions {
          height: 3,
          width: 4,
        },
        path: format!("artwork/{}/thumbnail", test::FINGERPRINT),
      }),
    );

    let html = PackageHtml {
      colophon: None,
      directory: Directory::new(),
      fingerprint: test::FINGERPRINT.parse().unwrap(),
      identifier: PackageIdentifier::Fingerprint(test::FINGERPRINT.parse().unwrap()),
      metadata: None,
      mounted: false,
      next: None,
      number: Some(1),
      prev: None,
      readme: None,
      revision: None,
      totals: Totals::default(),
    };

    assert_eq!(html.open_graph_image(), None);
    assert_eq!(html.open_graph_description(), None);
  }

  #[test]
  fn package() {
    let metadata = Metadata {
      language: Some("en".parse().unwrap()),
      package: Some(Package {
        colophon: Some("COLOPHON.md".parse().unwrap()),
        creator: Some("foo".parse().unwrap()),
        description: Some("bar".parse().unwrap()),
        homepage: Some("http://example.com".parse().unwrap()),
        time: Some("2024-01-01".parse().unwrap()),
        title: Some("baz".parse().unwrap()),
      }),
      readme: Some("README.md".parse().unwrap()),
      ..default()
    };

    let mut directory = Directory::new();
    directory.insert_entry("foo", Entry::file(Hash::bytes(b"foo"), 3));

    assert_eq!(
      PackageHtml {
        colophon: Some(test::HASH.parse().unwrap()),
        directory,
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        identifier: PackageIdentifier::Fingerprint(test::FINGERPRINT.parse().unwrap()),
        metadata: Some(metadata),
        mounted: false,
        next: None,
        number: Some(1),
        prev: None,
        readme: Some(test::HASH.parse().unwrap()),
        revision: None,
        totals: Totals {
          directories: 0,
          directory_size: 0,
          file_size: 3,
          files: 1,
        },
      }
      .to_string(),
      unindent(&format!(
        "
          <dl>
            <div>
              <dt>number</dt>
              <dd>
                <a href='/package/1'>1</a>
              </dd>
            </div>
            <div>
              <dt>fingerprint</dt>
              <dd>
                <a href='/package/{fingerprint}'><code>{fingerprint}</code></a>
              </dd>
            </div>
            <div>
              <dt>size</dt>
              <dd>
                3 B
              </dd>
            </div>
            <div>
              <dt>files</dt>
              <dd>
                <a href='/directory/{hash}'>1 file</a>
              </dd>
            </div>
            <div>
              <dt>readme</dt>
              <dd>
                <a href='/file/{hash}/README.md'>view</a>
              </dd>
            </div>
            <div>
              <dt>language</dt>
              <dd>
                English
              </dd>
            </div>
            <div>
              <dt>package</dt>
              <dd>
                <dl>
                  <div>
                    <dt>title</dt>
                    <dd>
                      baz
                    </dd>
                  </div>
                  <div>
                    <dt>creator</dt>
                    <dd>
                      foo
                    </dd>
                  </div>
                  <div>
                    <dt>time</dt>
                    <dd>
                      2024-01-01
                    </dd>
                  </div>
                  <div>
                    <dt>description</dt>
                    <dd>
                      bar
                    </dd>
                  </div>
                  <div>
                    <dt>colophon</dt>
                    <dd>
                      <a href='/file/{hash}/COLOPHON.md'>view</a>
                    </dd>
                  </div>
                  <div>
                    <dt>homepage</dt>
                    <dd>
                      <a href='http://example.com'>http://example.com</a>
                    </dd>
                  </div>
                </dl>
              </dd>
            </div>
          </dl>
          <table>
            <thead>
              <tr>
                <th>open</th>
                <th>download</th>
              </tr>
            </thead>
            <tbody>
              <tr>
                <td><a>foo</a></td>
                <td class=right><a href=/file/{foo} download=\"foo\">3 B</a></td>
              </tr>
            </tbody>
          </table>
        ",
        fingerprint = test::FINGERPRINT,
        foo = Hash::bytes(b"foo"),
        hash = test::HASH,
      )),
    );
  }

  #[test]
  fn video() {
    let metadata = Metadata {
      media: Some(Media::Video {
        items: vec![
          Item {
            content: Video {
              placeholder: None,
              duration: 225_000,
              path: "foo.mp4".parse().unwrap(),
              tracks: vec![
                Track {
                  codec: Some(Codec::H264),
                  info: Some(TrackInfo::Video {
                    bit_depth: 8,
                    chroma_subsampling: Some(ChromaSubsampling::Yuv420),
                    dimensions: Dimensions {
                      height: 1,
                      width: 2,
                    },
                    frames: 0,
                    orientation: Orientation::new(),
                  }),
                  size: 0,
                },
                Track {
                  codec: Some(Codec::Aac),
                  info: Some(TrackInfo::Audio {
                    channels: 2,
                    sample_rate: 44100,
                  }),
                  size: 0,
                },
              ],
              ty: Some(VideoType::Mp4),
            },
            title: None,
          },
          Item {
            content: Video {
              placeholder: Some(Image {
                dimensions: Dimensions {
                  height: 2,
                  width: 4,
                },
                ..Image::test("bar.png")
              }),
              ..Video::test("bar.mp4")
            },
            title: Some("bar".parse().unwrap()),
          },
          Item {
            content: Video {
              placeholder: Some(Image {
                dimensions: Dimensions {
                  height: 2,
                  width: 1,
                },
                ..Image::test("baz.png")
              }),
              ..Video::test("baz.mp4")
            },
            title: None,
          },
          Item::test("qux.mp4"),
        ],
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
    };

    assert_eq!(
      PackageHtml {
        colophon: None,
        directory: Directory::new(),
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        identifier: PackageIdentifier::Fingerprint(test::FINGERPRINT.parse().unwrap()),
        metadata: Some(metadata),
        mounted: false,
        next: None,
        number: Some(1),
        prev: None,
        readme: None,
        revision: None,
        totals: Totals {
          directories: 0,
          directory_size: 0,
          file_size: 3,
          files: 1,
        },
      }
      .to_string(),
      unindent(&format!(
        "
          <dl>
            <div>
              <dt>number</dt>
              <dd>
                <a href='/package/1'>1</a>
              </dd>
            </div>
            <div>
              <dt>fingerprint</dt>
              <dd>
                <a href='/package/{fingerprint}'><code>{fingerprint}</code></a>
              </dd>
            </div>
            <div>
              <dt>size</dt>
              <dd>
                3 B
              </dd>
            </div>
            <div>
              <dt>files</dt>
              <dd>
                <a href='/directory/{hash}'>1 file</a>
              </dd>
            </div>
            <div>
              <dt>media</dt>
              <dd>
                <a href='/package/{fingerprint}/media'>video</a>
              </dd>
            </div>
            <div>
              <dt>videos</dt>
              <dd>
                4
              </dd>
            </div>
            <div>
              <dt>duration</dt>
              <dd>
                3:48
              </dd>
            </div>
            <div>
              <dt>format</dt>
              <dd>
                <ol role=list>
                  <li>
                    MP4
                  </li>
                </ol>
              </dd>
            </div>
          </dl>
          <ol class=thumbnails>
            <li style=\"--aspect-ratio: 2 / 1\">
              <a href=/package/{fingerprint}/item/1>
                <div class=caption>
                  <time datetime=PT3M45S>3:45</time>
                </div>
              </a>
            </li>
            <li style=\"--aspect-ratio: 2 / 1\">
              <a href=/package/{fingerprint}/item/2>
                <img loading=lazy src=/media/video/{fingerprint}/item/2/placeholder/thumbnail>
                <div class=caption>
                  <span class=title>bar</span>
                  <time datetime=PT0M1S>0:01</time>
                </div>
              </a>
            </li>
            <li style=\"--aspect-ratio: 1 / 2\">
              <a href=/package/{fingerprint}/item/3>
                <img loading=lazy src=/media/video/{fingerprint}/item/3/placeholder/thumbnail>
                <div class=caption>
                  <time datetime=PT0M1S>0:01</time>
                </div>
              </a>
            </li>
            <li>
              <a href=/package/{fingerprint}/item/4>
                <div class=caption>
                  <time datetime=PT0M1S>0:01</time>
                </div>
              </a>
            </li>
          </ol>
        ",
        fingerprint = test::FINGERPRINT,
        hash = test::HASH,
      )),
    );
  }
}
