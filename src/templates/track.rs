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
