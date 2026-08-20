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
  fn title(&self) -> Option<&Component> {
    self.metadata.as_ref()?.title.as_deref()
  }
}

impl Page for PackageHtml {
  fn open_graph_description(&self) -> Option<String> {
    Some(self.metadata.as_ref()?.description.as_ref()?.to_string())
  }

  fn open_graph_image(&self) -> Option<OpenGraphImage> {
    let artwork = self.metadata.as_ref()?.artwork.as_ref()?;
    Some(OpenGraphImage {
      dimensions: artwork.dimensions,
      path: format!("artwork/{}", self.fingerprint),
    })
  }

  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/package.css")
  }

  fn title(&self) -> String {
    if let Some(title) = self.title() {
      format!("{title} · filepack")
    } else {
      format!("{} · filepack", self.fingerprint)
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
          Audio {
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
            title: "foo".parse().unwrap(),
            track: 1,
            tracks: 2,
            ty: AudioType::Flac,
          },
          Audio {
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
            title: "bar".parse().unwrap(),
            track: 2,
            tracks: 2,
            ty: AudioType::Flac,
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
            <dt>fingerprint</dt>
            <dd class=code>{fingerprint}</dd>
            <dt>size</dt>
            <dd>6 B</dd>
            <dt>files</dt>
            <dd><a href=/directory/{hash}>2 files</a></dd>
            <dt>media</dt>
            <dd><a href=/package/{fingerprint}/media>audio</a></dd>
            <dt>tracks</dt>
            <dd>2</dd>
            <dt>duration</dt>
            <dd>3:46</dd>
            <dt>format</dt>
            <dd>FLAC</dd>
          </dl>
          <ol>
            <li>
              <a href=/package/{fingerprint}/item/1>foo</a>
              <time datetime=PT3M45S>3:45</time>
            </li>
            <li>
              <a href=/package/{fingerprint}/item/2>bar</a>
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
          Audio {
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
            title: "foo".parse().unwrap(),
            track: 1,
            tracks: 2,
            ty: AudioType::Flac,
          },
          Audio {
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
            title: "bar".parse().unwrap(),
            track: 2,
            tracks: 2,
            ty: AudioType::Flac,
          },
          Audio {
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
            title: "baz".parse().unwrap(),
            track: 1,
            tracks: 1,
            ty: AudioType::Flac,
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
            <dt>fingerprint</dt>
            <dd class=code>{fingerprint}</dd>
            <dt>size</dt>
            <dd>9 B</dd>
            <dt>files</dt>
            <dd><a href=/directory/{hash}>3 files</a></dd>
            <dt>media</dt>
            <dd><a href=/package/{fingerprint}/media>audio</a></dd>
            <dt>tracks</dt>
            <dd>3</dd>
            <dt>duration</dt>
            <dd>0:03</dd>
            <dt>format</dt>
            <dd>FLAC</dd>
          </dl>
          <h2>disc 1</h2>
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
          <h2>disc 2</h2>
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
    let audio = Audio {
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
      title: "foo".parse().unwrap(),
      track: 1,
      tracks: 2,
      ty: AudioType::Flac,
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
            <dt>fingerprint</dt>
            <dd class=code>{fingerprint}</dd>
            <dt>size</dt>
            <dd>6 B</dd>
            <dt>files</dt>
            <dd><a href=/directory/{hash}>2 files</a></dd>
            <dt>media</dt>
            <dd><a href=/package/{fingerprint}/media>audio</a></dd>
            <dt>tracks</dt>
            <dd>2</dd>
            <dt>duration</dt>
            <dd>5124095576030431:00:15</dd>
            <dt>format</dt>
            <dd>FLAC</dd>
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
          Image {
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
          Image {
            alpha: false,
            bit_depth: 8,
            chroma_subsampling: None,
            color_type: ColorType::Rgb,
            dimensions: Dimensions::default(),
            orientation: Orientation::new(),
            path: "bar.jpg".parse().unwrap(),
            ty: ImageType::Jpeg,
          },
          Image {
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
            <dt>fingerprint</dt>
            <dd class=code>{fingerprint}</dd>
            <dt>size</dt>
            <dd>9 B</dd>
            <dt>files</dt>
            <dd><a href=/directory/{hash}>3 files</a></dd>
            <dt>media</dt>
            <dd><a href=/package/{fingerprint}/media>image</a></dd>
            <dt>images</dt>
            <dd>3</dd>
            <dt>format</dt>
            <dd>PNG</dd>
            <dd>JPEG</dd>
          </dl>
          <ul class=thumbnails>
            <li>
              <a href=/package/{fingerprint}/item/1>
                <img loading=lazy src=/media/image/{fingerprint}/item/1>
              </a>
            </li>
            <li>
              <a href=/package/{fingerprint}/item/2>
                <img loading=lazy src=/media/image/{fingerprint}/item/2>
              </a>
            </li>
            <li>
              <a href=/package/{fingerprint}/item/3>
                <img loading=lazy src=/media/image/{fingerprint}/item/3>
              </a>
            </li>
          </ul>
        ",
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
          orientation: Orientation::new(),
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
          height: 1,
          width: 2,
        },
        path: format!("artwork/{}", test::FINGERPRINT),
      }),
    );

    assert_eq!(html.open_graph_description(), Some("bar".into()));

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
            <dt>fingerprint</dt>
            <dd class=code>{fingerprint}</dd>
            <dt>size</dt>
            <dd>3 B</dd>
            <dt>files</dt>
            <dd><a href=/directory/{hash}>1 files</a></dd>
            <dt>readme</dt>
            <dd><a href='/file/{hash}/README.md'>view</a></dd>
            <dt>package</dt>
            <dd>
              <dl>
                <dt>title</dt>
                <dd>baz</dd>
                <dt>creator</dt>
                <dd>foo</dd>
                <dt>time</dt>
                <dd>2024-01-01</dd>
                <dt>description</dt>
                <dd>bar</dd>
                <dt>colophon</dt>
                <dd><a href='/file/{hash}/COLOPHON.md'>view</a></dd>
                <dt>homepage</dt>
                <dd><a href='http://example.com'>http://example.com</a></dd>
              </dl>
            </dd>
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
        items: vec![Video {
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
            <dt>fingerprint</dt>
            <dd class=code>{fingerprint}</dd>
            <dt>size</dt>
            <dd>3 B</dd>
            <dt>files</dt>
            <dd><a href=/directory/{hash}>1 files</a></dd>
            <dt>media</dt>
            <dd><a href=/package/{fingerprint}/media>video</a></dd>
            <dt>videos</dt>
            <dd>1</dd>
            <dt>duration</dt>
            <dd>3:45</dd>
            <dt>format</dt>
            <dd>MP4</dd>
          </dl>
          <ol>
            <li>
              <a href=/package/{fingerprint}/item/1>foo.mp4</a>
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
