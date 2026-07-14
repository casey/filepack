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
            filename: "foo.flac".parse().unwrap(),
            sample_count: 9_922_500,
            sample_rate: 44100,
            title: "foo".parse().unwrap(),
            ty: AudioType::Flac,
          },
          Track {
            album: "qux".parse().unwrap(),
            artist: "baz".parse().unwrap(),
            filename: "bar.flac".parse().unwrap(),
            sample_count: 44100,
            sample_rate: 44100,
            title: "bar".parse().unwrap(),
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
          </dl>
          <ul>
            <li>
              <a href=/package/{fingerprint}/1>foo</a>
              3:45
            </li>
            <li>
              <a href=/package/{fingerprint}/2>bar</a>
              0:01
            </li>
          </ul>
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
      filename: "foo.flac".parse().unwrap(),
      sample_count: u64::MAX,
      sample_rate: 1,
      title: "foo".parse().unwrap(),
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
          </dl>
          <ul>
            <li>
              <a href=/package/{fingerprint}/1>foo</a>
              5124095576030431:00:15
            </li>
            <li>
              <a href=/package/{fingerprint}/2>foo</a>
              5124095576030431:00:15
            </li>
          </ul>
        ",
        fingerprint = test::FINGERPRINT,
        hash = test::HASH,
      )),
    );
  }
}
