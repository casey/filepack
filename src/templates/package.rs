use super::*;

#[derive(Boilerplate)]
pub struct PackageHtml {
  pub colophon: Option<Hash>,
  pub directory: Directory,
  pub fingerprint: Fingerprint,
  pub metadata: Option<Metadata>,
  pub mounted: bool,
  pub readme: Option<Hash>,
  pub totals: Totals,
}

impl PackageHtml {
  fn title(&self) -> Option<&str> {
    self.metadata.as_ref()?.title.as_deref()
  }
}

impl Page for PackageHtml {
  fn open_graph_description(&self) -> Option<String> {
    Some(self.metadata.as_ref()?.description.as_ref()?.to_string())
  }

  fn open_graph_image(&self) -> Option<OpenGraphImage> {
    OpenGraphImage::artwork(self.metadata.as_ref()?, self.fingerprint)
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
              album: "qux".parse().unwrap(),
              artist: "baz".parse().unwrap(),
              channels: 2,
              disc: 1,
              discs: 1,
              path: "foo.flac".parse().unwrap(),
              sample_bits: Some(16),
              sample_rate: 44100,
              samples: 9_922_500,
              size: 0,
              track: 1,
              tracks: 2,
              ty: AudioType::Flac,
            },
            title: Some("foo".parse().unwrap()),
          },
          Item {
            content: Audio {
              album: "qux".parse().unwrap(),
              artist: "baz".parse().unwrap(),
              channels: 2,
              disc: 1,
              discs: 1,
              path: "bar.flac".parse().unwrap(),
              sample_bits: Some(24),
              sample_rate: 96000,
              samples: 96000,
              size: 0,
              track: 2,
              tracks: 2,
              ty: AudioType::Flac,
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
        metadata: Some(metadata),
        mounted: false,
        readme: None,
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
          <h1 class=code>{fingerprint}</h1>
          <dl>
            <div>
              <dt>fingerprint</dt>
              <dd class=code>{fingerprint}</dd>
            </div>
            <div>
              <dt>size</dt>
              <dd>6 B</dd>
            </div>
            <div>
              <dt>files</dt>
              <dd><a href=/directory/{hash}>2 files</a></dd>
            </div>
            <div>
              <dt>media</dt>
              <dd><a href=/package/{fingerprint}/media>audio</a></dd>
            </div>
            <div>
              <dt>tracks</dt>
              <dd>2</dd>
            </div>
            <div>
              <dt>duration</dt>
              <dd>3:46</dd>
            </div>
            <div>
              <dt>format</dt>
              <dd>
                FLAC
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
  fn audio_multiple_discs() {
    let metadata = Metadata {
      media: Some(Media::Audio {
        items: vec![
          Item {
            content: Audio {
              album: "qux".parse().unwrap(),
              artist: "baz".parse().unwrap(),
              channels: 2,
              disc: 1,
              discs: 2,
              path: "foo.flac".parse().unwrap(),
              sample_bits: Some(16),
              sample_rate: 44100,
              samples: 44100,
              size: 0,
              track: 1,
              tracks: 2,
              ty: AudioType::Flac,
            },
            title: Some("foo".parse().unwrap()),
          },
          Item {
            content: Audio {
              album: "qux".parse().unwrap(),
              artist: "baz".parse().unwrap(),
              channels: 2,
              disc: 1,
              discs: 2,
              path: "bar.flac".parse().unwrap(),
              sample_bits: Some(16),
              sample_rate: 44100,
              samples: 44100,
              size: 0,
              track: 2,
              tracks: 2,
              ty: AudioType::Flac,
            },
            title: Some("bar".parse().unwrap()),
          },
          Item {
            content: Audio {
              album: "qux".parse().unwrap(),
              artist: "baz".parse().unwrap(),
              channels: 2,
              disc: 2,
              discs: 2,
              path: "baz.flac".parse().unwrap(),
              sample_bits: Some(16),
              sample_rate: 44100,
              samples: 44100,
              size: 0,
              track: 1,
              tracks: 1,
              ty: AudioType::Flac,
            },
            title: Some("baz".parse().unwrap()),
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
        metadata: Some(metadata),
        mounted: false,
        readme: None,
        totals: Totals {
          directories: 0,
          directory_size: 0,
          file_size: 9,
          files: 3,
        },
      }
      .to_string(),
      unindent(&format!(
        "
          <h1 class=code>{fingerprint}</h1>
          <dl>
            <div>
              <dt>fingerprint</dt>
              <dd class=code>{fingerprint}</dd>
            </div>
            <div>
              <dt>size</dt>
              <dd>9 B</dd>
            </div>
            <div>
              <dt>files</dt>
              <dd><a href=/directory/{hash}>3 files</a></dd>
            </div>
            <div>
              <dt>media</dt>
              <dd><a href=/package/{fingerprint}/media>audio</a></dd>
            </div>
            <div>
              <dt>tracks</dt>
              <dd>3</dd>
            </div>
            <div>
              <dt>duration</dt>
              <dd>0:03</dd>
            </div>
            <div>
              <dt>format</dt>
              <dd>
                FLAC
              </dd>
            </div>
          </dl>
          <h2>Disc 1</h2>
          <ol>
            <li>
              <a href=/package/{fingerprint}/item/1>foo</a>
              <time datetime=PT0M1S>0:01</time>
            </li>
            <li>
              <a href=/package/{fingerprint}/item/2>bar</a>
              <time datetime=PT0M1S>0:01</time>
            </li>
          </ol>
          <h2>Disc 2</h2>
          <ol>
            <li>
              <a href=/package/{fingerprint}/item/3>baz</a>
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
        album: "qux".parse().unwrap(),
        artist: "baz".parse().unwrap(),
        channels: 2,
        disc: 1,
        discs: 1,
        path: "foo.flac".parse().unwrap(),
        sample_bits: Some(16),
        sample_rate: 1,
        samples: u64::MAX,
        size: 0,
        track: 1,
        tracks: 2,
        ty: AudioType::Flac,
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
        metadata: Some(metadata),
        mounted: false,
        readme: None,
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
          <h1 class=code>{fingerprint}</h1>
          <dl>
            <div>
              <dt>fingerprint</dt>
              <dd class=code>{fingerprint}</dd>
            </div>
            <div>
              <dt>size</dt>
              <dd>6 B</dd>
            </div>
            <div>
              <dt>files</dt>
              <dd><a href=/directory/{hash}>2 files</a></dd>
            </div>
            <div>
              <dt>media</dt>
              <dd><a href=/package/{fingerprint}/media>audio</a></dd>
            </div>
            <div>
              <dt>tracks</dt>
              <dd>2</dd>
            </div>
            <div>
              <dt>duration</dt>
              <dd>5124095576030431:00:15</dd>
            </div>
            <div>
              <dt>format</dt>
              <dd>
                FLAC
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
              color_type: ColorType::Rgb,
              dimensions: Dimensions {
                height: 1,
                width: 2,
              },
              orientation: Orientation::new(),
              path: "foo.png".parse().unwrap(),
              ty: ImageType::Png,
            },
            title: None,
          },
          Item {
            content: Image {
              alpha: false,
              bit_depth: 8,
              chroma_subsampling: None,
              color_type: ColorType::Rgb,
              dimensions: Dimensions::default(),
              orientation: Orientation::new(),
              path: "bar.jpg".parse().unwrap(),
              ty: ImageType::Jpeg,
            },
            title: None,
          },
          Item {
            content: Image {
              alpha: false,
              bit_depth: 8,
              chroma_subsampling: None,
              color_type: ColorType::Rgb,
              dimensions: Dimensions {
                height: 1,
                width: 2,
              },
              orientation: Orientation::new(),
              path: "baz.png".parse().unwrap(),
              ty: ImageType::Png,
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
        metadata: Some(metadata),
        mounted: false,
        readme: None,
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
          <h1 class=code>{fingerprint}</h1>
          <dl>
            <div>
              <dt>fingerprint</dt>
              <dd class=code>{fingerprint}</dd>
            </div>
            <div>
              <dt>size</dt>
              <dd>9 B</dd>
            </div>
            <div>
              <dt>files</dt>
              <dd><a href=/directory/{hash}>3 files</a></dd>
            </div>
            <div>
              <dt>publisher</dt>
              <dd>qux</dd>
            </div>
            <div>
              <dt>media</dt>
              <dd><a href=/package/{fingerprint}/media>image</a></dd>
            </div>
            <div>
              <dt>images</dt>
              <dd>3</dd>
            </div>
            <div>
              <dt>format</dt>
              <dd>
                PNG
                JPEG
              </dd>
            </div>
          </dl>
          <ul class=thumbnails>
            <li style="--aspect-ratio: 2 / 1">
              <a href=/package/{fingerprint}/item/1>
                <img loading=lazy src=/media/image/{fingerprint}/item/1/thumbnail>
              </a>
            </li>
            <li style="--aspect-ratio: 1 / 1">
              <a href=/package/{fingerprint}/item/2>
                <img loading=lazy src=/media/image/{fingerprint}/item/2/thumbnail>
              </a>
            </li>
            <li style="--aspect-ratio: 2 / 1">
              <a href=/package/{fingerprint}/item/3>
                <img loading=lazy src=/media/image/{fingerprint}/item/3/thumbnail>
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
  fn open_graph_metadata() {
    let html = PackageHtml {
      colophon: None,
      directory: Directory::new(),
      fingerprint: test::FINGERPRINT.parse().unwrap(),
      metadata: Some(Metadata {
        artwork: Some(Image {
          alpha: false,
          bit_depth: 8,
          chroma_subsampling: None,
          color_type: ColorType::Rgb,
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
          orientation: Orientation {
            mirrored: false,
            rotation: Rotation::R90,
          },
          path: "foo.png".parse().unwrap(),
          ty: ImageType::Png,
        }),
        description: Some("bar".parse().unwrap()),
        ..default()
      }),
      mounted: false,
      readme: None,
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
          color_type: ColorType::Rgb,
          dimensions: Dimensions {
            height: 3,
            width: 4,
          },
          orientation: Orientation::default(),
          path: "thumbnails/foo.jpg".parse().unwrap(),
          ty: ImageType::Jpeg,
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
      metadata: None,
      mounted: false,
      readme: None,
      totals: Totals::default(),
    };

    assert_eq!(html.open_graph_image(), None);
    assert_eq!(html.open_graph_description(), None);
  }

  #[test]
  fn package() {
    let metadata = Metadata {
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
        metadata: Some(metadata),
        mounted: false,
        readme: Some(test::HASH.parse().unwrap()),
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
          <h1 class=code>{fingerprint}</h1>
          <dl>
            <div>
              <dt>fingerprint</dt>
              <dd class=code>{fingerprint}</dd>
            </div>
            <div>
              <dt>size</dt>
              <dd>3 B</dd>
            </div>
            <div>
              <dt>files</dt>
              <dd><a href=/directory/{hash}>1 files</a></dd>
            </div>
            <div>
              <dt>readme</dt>
              <dd><a href='/file/{hash}/README.md'>view</a></dd>
            </div>
            <div>
              <dt>package</dt>
              <dd>
                <dl>
                  <div>
                    <dt>title</dt>
                    <dd>baz</dd>
                  </div>
                  <div>
                    <dt>creator</dt>
                    <dd>foo</dd>
                  </div>
                  <div>
                    <dt>time</dt>
                    <dd>2024-01-01</dd>
                  </div>
                  <div>
                    <dt>description</dt>
                    <dd>bar</dd>
                  </div>
                  <div>
                    <dt>colophon</dt>
                    <dd><a href='/file/{hash}/COLOPHON.md'>view</a></dd>
                  </div>
                  <div>
                    <dt>homepage</dt>
                    <dd><a href='http://example.com'>http://example.com</a></dd>
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
        items: vec![Item {
          content: Video {
            placeholder: None,
            duration: 225_000,
            path: "foo.mp4".parse().unwrap(),
            tracks: vec![
              Track {
                codec: Codec::H264,
                info: TrackInfo::Video {
                  bit_depth: 8,
                  chroma_subsampling: ChromaSubsampling::Yuv420,
                  dimensions: Dimensions {
                    height: 1,
                    width: 2,
                  },
                  frames: 0,
                  orientation: Orientation::new(),
                },
                size: 0,
              },
              Track {
                codec: Codec::Aac,
                info: TrackInfo::Audio {
                  channels: 2,
                  sample_rate: 44100,
                },
                size: 0,
              },
            ],
            ty: VideoType::Mp4,
          },
          title: None,
        }],
      }),
      ..default()
    };

    assert_eq!(
      PackageHtml {
        colophon: None,
        directory: Directory::new(),
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        metadata: Some(metadata),
        mounted: false,
        readme: None,
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
          <h1 class=code>{fingerprint}</h1>
          <dl>
            <div>
              <dt>fingerprint</dt>
              <dd class=code>{fingerprint}</dd>
            </div>
            <div>
              <dt>size</dt>
              <dd>3 B</dd>
            </div>
            <div>
              <dt>files</dt>
              <dd><a href=/directory/{hash}>1 files</a></dd>
            </div>
            <div>
              <dt>media</dt>
              <dd><a href=/package/{fingerprint}/media>video</a></dd>
            </div>
            <div>
              <dt>videos</dt>
              <dd>1</dd>
            </div>
            <div>
              <dt>duration</dt>
              <dd>3:45</dd>
            </div>
            <div>
              <dt>format</dt>
              <dd>
                MP4
              </dd>
            </div>
          </dl>
          <ol>
            <li>
              <a href=/package/{fingerprint}/item/1>Video 1</a>
              <time datetime=PT3M45S>3:45</time>
            </li>
          </ol>
        ",
        fingerprint = test::FINGERPRINT,
        hash = test::HASH,
      )),
    );
  }
}
