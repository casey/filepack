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
    format!("{} · filepack", self.track().title)
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
          album: "qux".parse().unwrap(),
          artist: "baz".parse().unwrap(),
          filename: "foo.flac".parse().unwrap(),
          sample_count: 9_922_500,
          sample_rate: 44100,
          title: "foo".parse().unwrap(),
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
          <div class=info>
            <div class=title>foo</div>
            <div class=artist>baz</div>
            <div class=album>qux</div>
            <div class=duration>3:45</div>
          </div>
          <audio controls src=/media/audio/{fingerprint}/track/1></audio>
        ",
        fingerprint = test::FINGERPRINT,
      )),
    );
  }
}
