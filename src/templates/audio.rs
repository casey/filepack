use super::*;

#[derive(Boilerplate)]
pub(crate) struct AudioHtml {
  pub(crate) audio: usize,
  pub(crate) fingerprint: Fingerprint,
  pub(crate) metadata: Metadata,
}

impl AudioHtml {
  fn audio(&self) -> &Item<Audio> {
    let Media::Audio { items } = self.metadata.media.as_ref().unwrap() else {
      unreachable!();
    };

    &items[self.audio]
  }
}

impl Page for AudioHtml {
  fn open_graph_image(&self) -> Option<OpenGraphImage> {
    OpenGraphImage::artwork(&self.metadata, self.fingerprint)
  }

  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/audio.css")
  }

  fn title(&self) -> String {
    format!("{} · filepack", self.audio().display_title())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn audio() {
    let metadata = Metadata {
      media: Some(Media::Audio {
        items: vec![Item {
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
            tracks: 1,
            ty: AudioType::Flac,
          },
          title: Some("foo".parse().unwrap()),
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
          <audio controls src=/media/audio/{fingerprint}/item/1></audio>
        ",
        fingerprint = test::FINGERPRINT,
      )),
    );
  }

  #[test]
  fn open_graph_image() {
    let html = AudioHtml {
      audio: 0,
      fingerprint: test::FINGERPRINT.parse().unwrap(),
      metadata: Metadata {
        artwork: Some(Image::test("foo.png")),
        ..default()
      },
    };

    assert_eq!(
      html.open_graph_image(),
      Some(OpenGraphImage {
        dimensions: Dimensions {
          height: 1,
          width: 1
        },
        path: format!("artwork/{}", test::FINGERPRINT),
      }),
    );

    let html = AudioHtml {
      audio: 0,
      fingerprint: test::FINGERPRINT.parse().unwrap(),
      metadata: default(),
    };

    assert_eq!(html.open_graph_image(), None);
  }
}
