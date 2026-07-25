use super::*;

#[derive(Boilerplate)]
pub(crate) struct MediaHtml {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) metadata: Metadata,
}

impl MediaHtml {
  fn title(&self) -> Option<&Component> {
    self.metadata.title.as_deref()
  }
}

impl Page for MediaHtml {
  fn title(&self) -> String {
    if let Some(title) = self.title() {
      format!("{title} media · filepack")
    } else {
      format!("{} media · filepack", self.fingerprint)
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
        tracks: vec![Audio {
          album: "qux".parse().unwrap(),
          artist: "baz".parse().unwrap(),
          channels: 2,
          disc: 1,
          discs: 1,
          filename: "foo.flac".parse().unwrap(),
          sample_bits: 16,
          sample_rate: 44100,
          samples: 9_922_500,
          title: "bar".parse().unwrap(),
          track: 1,
          tracks: 2,
          ty: AudioType::Flac,
        }],
      }),
      title: Some("foo".parse().unwrap()),
      ..default()
    };

    assert_eq!(
      MediaHtml {
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        metadata,
      }
      .to_string(),
      unindent(&format!(
        "
          <h1><a href=/package/{fingerprint}>foo</a></h1>
          <ol>
            <li>
              <a href=/package/{fingerprint}/item/1>foo.flac</a>
              <dl>
                <dt>filename</dt>
                <dd>
                  foo.flac
                </dd>
                <dt>title</dt>
                <dd>
                  bar
                </dd>
                <dt>artist</dt>
                <dd>
                  baz
                </dd>
                <dt>album</dt>
                <dd>
                  qux
                </dd>
                <dt>disc</dt>
                <dd>
                  1 of 1
                </dd>
                <dt>track</dt>
                <dd>
                  1 of 2
                </dd>
                <dt>duration</dt>
                <dd>
                  3:45
                </dd>
                <dt>format</dt>
                <dd>
                  FLAC · 16-bit 44.1 kHz stereo · lossless
                </dd>
                <dt>samples</dt>
                <dd>
                  9922500
                </dd>
              </dl>
            </li>
          </ol>
        ",
        fingerprint = test::FINGERPRINT,
      )),
    );
  }

  #[test]
  fn image() {
    let metadata = Metadata {
      media: Some(Media::Image {
        images: vec![Image {
          dimensions: Dimensions {
            height: 1,
            width: 2,
          },
          filename: "foo.png".parse().unwrap(),
          ty: ImageType::Png,
        }],
      }),
      ..default()
    };

    assert_eq!(
      MediaHtml {
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        metadata,
      }
      .to_string(),
      unindent(&format!(
        "
          <h1 class=code><a href=/package/{fingerprint}>{fingerprint}</a></h1>
          <ol>
            <li>
              <a href=/package/{fingerprint}/item/1>foo.png</a>
              <dl>
                <dt>filename</dt>
                <dd>
                  foo.png
                </dd>
                <dt>type</dt>
                <dd>
                  PNG
                </dd>
                <dt>dimensions</dt>
                <dd>
                  2×1
                </dd>
              </dl>
            </li>
          </ol>
        ",
        fingerprint = test::FINGERPRINT,
      )),
    );
  }

  #[test]
  fn video() {
    let metadata = Metadata {
      media: Some(Media::Video {
        videos: vec![Video {
          duration: 225_000,
          filename: "foo.mp4".parse().unwrap(),
          tracks: vec![
            Track {
              codec: Codec::H264,
              info: TrackInfo::Video {
                bit_depth: Some(8),
                dimensions: Dimensions {
                  height: 1,
                  width: 2,
                },
                frames: 0,
              },
              size: 0,
            },
            Track {
              codec: Codec::Aac,
              info: TrackInfo::Audio,
              size: 0,
            },
          ],
          ty: VideoType::Mp4,
        }],
      }),
      ..default()
    };

    assert_eq!(
      MediaHtml {
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        metadata,
      }
      .to_string(),
      unindent(&format!(
        "
          <h1 class=code><a href=/package/{fingerprint}>{fingerprint}</a></h1>
          <ol>
            <li>
              <a href=/package/{fingerprint}/item/1>foo.mp4</a>
              <dl>
                <dt>filename</dt>
                <dd>
                  foo.mp4
                </dd>
                <dt>type</dt>
                <dd>
                  MP4
                </dd>
                <dt>duration</dt>
                <dd>
                  3:45
                </dd>
                <dt>tracks</dt>
                <dd>
                  <ol>
                    <li>
                      <dl>
                        <dt>type</dt>
                        <dd>
                          video
                        </dd>
                        <dt>codec</dt>
                        <dd>
                          H.264
                        </dd>
                        <dt>dimensions</dt>
                        <dd>
                          2×1
                        </dd>
                        <dt>frames</dt>
                        <dd>
                          0
                        </dd>
                        <dt>bit depth</dt>
                        <dd>
                          8-bit
                        </dd>
                        <dt>size</dt>
                        <dd>
                          0 B
                        </dd>
                      </dl>
                    </li>
                    <li>
                      <dl>
                        <dt>type</dt>
                        <dd>
                          audio
                        </dd>
                        <dt>codec</dt>
                        <dd>
                          AAC
                        </dd>
                        <dt>size</dt>
                        <dd>
                          0 B
                        </dd>
                      </dl>
                    </li>
                  </ol>
                </dd>
              </dl>
            </li>
          </ol>
        ",
        fingerprint = test::FINGERPRINT,
      )),
    );
  }
}
