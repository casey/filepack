use super::*;

#[derive(Boilerplate)]
pub(crate) struct TrackHtml {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) metadata: Metadata,
  pub(crate) track: usize,
}

impl TrackHtml {
  fn track(&self) -> &Track {
    let Media::Audio { tracks } = self.metadata.media.as_ref().unwrap() else {
      unreachable!();
    };

    &tracks[self.track]
  }
}

impl Page for TrackHtml {
  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/track.css")
  }

  fn title(&self) -> String {
    format!("{} · filepack", self.track().title())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn duration() {
    let metadata = Metadata {
      media: Some(Media::Audio {
        tracks: vec![Track {
          filename: "foo.flac".parse().unwrap(),
          sample_count: 9_922_500,
          sample_rate: 44100,
          title: Some("foo".into()),
          ty: AudioType::Flac,
        }],
      }),
      ..default()
    };

    assert_eq!(
      TrackHtml {
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        metadata,
        track: 0,
      }
      .to_string(),
      unindent(&format!(
        "
          <img src=/artwork/{fingerprint}>
          <audio controls src=/media/audio/{fingerprint}/track/1></audio>
          <dl>
            <dt>duration</dt>
            <dd>3:45</dd>
          </dl>
        ",
        fingerprint = test::FINGERPRINT,
      )),
    );
  }
}
