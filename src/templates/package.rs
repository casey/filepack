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
  use super::*;

  #[test]
  fn audio() {
    let metadata = Metadata {
      media: Some(Media::Audio {
        tracks: vec![
          Track {
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
          Track {
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
          <h1>{fingerprint}</h1>
          <dl>
            <dt>fingerprint</dt>
            <dd>{fingerprint}</dd>
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
            <dd>FLAC 16-bit 44.1 kHz stereo, lossless</dd>
            <dd>FLAC 24-bit 96 kHz stereo, lossless</dd>
          </dl>
          <ol>
            <li>
              <a href=/package/{fingerprint}/1>foo</a>
              3:45
            </li>
            <li>
              <a href=/package/{fingerprint}/2>bar</a>
              0:01
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
          Track {
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
          Track {
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
          Track {
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
          <h1>{fingerprint}</h1>
          <dl>
            <dt>fingerprint</dt>
            <dd>{fingerprint}</dd>
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
            <dd>FLAC 16-bit 44.1 kHz stereo, lossless</dd>
          </dl>
          <h2>disc 1</h2>
          <ol>
            <li>
              <a href=/package/{fingerprint}/1>foo</a>
              0:01
            </li>
            <li>
              <a href=/package/{fingerprint}/2>bar</a>
              0:01
            </li>
          </ol>
          <h2>disc 2</h2>
          <ol>
            <li>
              <a href=/package/{fingerprint}/3>baz</a>
              0:01
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
    let track = Track {
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
        tracks: vec![track.clone(), track],
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
          <h1>{fingerprint}</h1>
          <dl>
            <dt>fingerprint</dt>
            <dd>{fingerprint}</dd>
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
            <dd>FLAC 16-bit 0.001 kHz stereo, lossless</dd>
          </dl>
          <ol>
            <li>
              <a href=/package/{fingerprint}/1>foo</a>
              5124095576030431:00:15
            </li>
            <li>
              <a href=/package/{fingerprint}/2>foo</a>
              5124095576030431:00:15
            </li>
          </ol>
        ",
        fingerprint = test::FINGERPRINT,
        hash = test::HASH,
      )),
    );
  }
}
