use super::*;

#[derive(Boilerplate)]
pub struct PackageHtml {
  pub fingerprint: Fingerprint,
  pub metadata: Option<Metadata>,
  pub totals: Totals,
}

impl PackageHtml {
  fn title(&self) -> Option<&Component> {
    self.metadata.as_ref()?.title.as_deref()
  }
}

impl Page for PackageHtml {
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
        tracks: vec![
          Audio {
            album: "qux".parse().unwrap(),
            artist: "baz".parse().unwrap(),
            channels: 2,
            disc: 1,
            discs: 1,
            filename: "foo.flac".parse().unwrap(),
            sample_bits: 16,
            sample_rate: 44100,
            samples: 9_922_500,
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
            filename: "bar.flac".parse().unwrap(),
            sample_bits: 24,
            sample_rate: 96000,
            samples: 96000,
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
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        metadata: Some(metadata),
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
            <dd>audio</dd>
            <dt>tracks</dt>
            <dd>2</dd>
            <dt>duration</dt>
            <dd>3:46</dd>
            <dt>format</dt>
            <dd>FLAC · 16-bit 44.1 kHz stereo · lossless</dd>
            <dd>FLAC · 24-bit 96 kHz stereo · lossless</dd>
          </dl>
          <ol>
            <li>
              <a href=/package/{fingerprint}/1>foo</a>
              <time datetime=PT3M45S>3:45</time>
            </li>
            <li>
              <a href=/package/{fingerprint}/2>bar</a>
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
        tracks: vec![
          Audio {
            album: "qux".parse().unwrap(),
            artist: "baz".parse().unwrap(),
            channels: 2,
            disc: 1,
            discs: 2,
            filename: "foo.flac".parse().unwrap(),
            sample_bits: 16,
            sample_rate: 44100,
            samples: 44100,
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
            filename: "bar.flac".parse().unwrap(),
            sample_bits: 16,
            sample_rate: 44100,
            samples: 44100,
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
            filename: "baz.flac".parse().unwrap(),
            sample_bits: 16,
            sample_rate: 44100,
            samples: 44100,
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
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        metadata: Some(metadata),
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
            <dd>audio</dd>
            <dt>tracks</dt>
            <dd>3</dd>
            <dt>duration</dt>
            <dd>0:03</dd>
            <dt>format</dt>
            <dd>FLAC · 16-bit 44.1 kHz stereo · lossless</dd>
          </dl>
          <h2>disc 1</h2>
          <ol>
            <li>
              <a href=/package/{fingerprint}/1>foo</a>
              <time datetime=PT0M1S>0:01</time>
            </li>
            <li>
              <a href=/package/{fingerprint}/2>bar</a>
              <time datetime=PT0M1S>0:01</time>
            </li>
          </ol>
          <h2>disc 2</h2>
          <ol>
            <li>
              <a href=/package/{fingerprint}/3>baz</a>
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
      filename: "foo.flac".parse().unwrap(),
      sample_bits: 16,
      sample_rate: 1,
      samples: u64::MAX,
      title: "foo".parse().unwrap(),
      track: 1,
      tracks: 2,
      ty: AudioType::Flac,
    };

    let metadata = Metadata {
      media: Some(Media::Audio {
        tracks: vec![audio.clone(), audio],
      }),
      ..default()
    };

    assert_eq!(
      PackageHtml {
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        metadata: Some(metadata),
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
            <dd>audio</dd>
            <dt>tracks</dt>
            <dd>2</dd>
            <dt>duration</dt>
            <dd>5124095576030431:00:15</dd>
            <dt>format</dt>
            <dd>FLAC · 16-bit 0.001 kHz stereo · lossless</dd>
          </dl>
          <ol>
            <li>
              <a href=/package/{fingerprint}/1>foo</a>
              <time datetime=PT5124095576030431H0M15S>5124095576030431:00:15</time>
            </li>
            <li>
              <a href=/package/{fingerprint}/2>foo</a>
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
        images: vec![
          Image {
            dimensions: Dimensions {
              height: 1,
              width: 2,
            },
            filename: "foo.png".parse().unwrap(),
            ty: ImageType::Png,
          },
          Image {
            dimensions: Dimensions::default(),
            filename: "bar.jpg".parse().unwrap(),
            ty: ImageType::Jpeg,
          },
          Image {
            dimensions: Dimensions {
              height: 1,
              width: 2,
            },
            filename: "baz.png".parse().unwrap(),
            ty: ImageType::Png,
          },
        ],
      }),
      ..default()
    };

    assert_eq!(
      PackageHtml {
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        metadata: Some(metadata),
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
            <dd>image</dd>
            <dt>images</dt>
            <dd>3</dd>
            <dt>format</dt>
            <dd>PNG</dd>
            <dd>JPEG</dd>
            <dt>resolution</dt>
            <dd>0×0–2×1</dd>
          </dl>
          <ul class=thumbnails>
            <li>
              <a href=/package/{fingerprint}/1>
                <img loading=lazy src=/media/image/{fingerprint}/item/1>
              </a>
            </li>
            <li>
              <a href=/package/{fingerprint}/2>
                <img loading=lazy src=/media/image/{fingerprint}/item/2>
              </a>
            </li>
            <li>
              <a href=/package/{fingerprint}/3>
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
  fn video() {
    let metadata = Metadata {
      media: Some(Media::Video {
        videos: vec![Video {
          duration: 0,
          filename: "foo.mp4".parse().unwrap(),
          tracks: vec![
            Track {
              codec: Codec::H264,
              info: TrackInfo::Video {
                dimensions: Dimensions {
                  height: 1,
                  width: 2,
                },
              },
            },
            Track {
              codec: Codec::Aac,
              info: TrackInfo::Audio,
            },
          ],
          ty: VideoType::Mp4,
        }],
      }),
      ..default()
    };

    assert_eq!(
      PackageHtml {
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        metadata: Some(metadata),
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
            <dd>video</dd>
            <dt>videos</dt>
            <dd>1</dd>
            <dt>format</dt>
            <dd>MP4 · H.264 · AAC</dd>
            <dt>resolution</dt>
            <dd>2×1</dd>
          </dl>
          <ol>
            <li><a href=/package/{fingerprint}/1>foo.mp4</a></li>
          </ol>
        ",
        fingerprint = test::FINGERPRINT,
        hash = test::HASH,
      )),
    );
  }
}
