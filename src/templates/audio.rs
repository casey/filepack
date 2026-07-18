use super::*;

#[derive(Boilerplate)]
pub(crate) struct AudioHtml {
  pub(crate) audio: usize,
  pub(crate) fingerprint: Fingerprint,
  pub(crate) metadata: Metadata,
}

impl AudioHtml {
  fn audio(&self) -> &Audio {
    let Media::Audio { tracks } = self.metadata.media.as_ref().unwrap() else {
      unreachable!();
    };

    &tracks[self.audio]
  }
}

impl Page for AudioHtml {
  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/audio.css")
  }

  fn title(&self) -> String {
    format!("{} · filepack", self.audio().title)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

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
          title: "foo".parse().unwrap(),
          track: 1,
          tracks: 1,
          ty: AudioType::Flac,
        }],
      }),
      ..default()
    };

    assert_eq!(
      AudioHtml {
        audio: 0,
        fingerprint: test::FINGERPRINT.parse().unwrap(),
        metadata,
      }
      .to_string(),
      unindent(&format!(
        "
          <img src=/artwork/{fingerprint}>
          <div class=info>
            <div class=title>foo</div>
            <div class=artist>baz</div>
            <div class=album>qux</div>
          </div>
          <audio controls src=/media/audio/{fingerprint}/track/1></audio>
        ",
        fingerprint = test::FINGERPRINT,
      )),
    );
  }
}
