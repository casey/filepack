use super::*;

#[derive(Boilerplate)]
pub(crate) struct MediaHtml {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) metadata: Metadata,
}

impl MediaHtml {
  fn title(&self) -> Option<&str> {
    self.metadata.title.as_deref()
  }
}

impl Page for MediaHtml {
  fn stylesheet(&self) -> Option<&'static str> {
    Some("/static/media.css")
  }

  fn title(&self) -> String {
    if let Some(title) = self.title() {
      format!("{title} media · filepack")
    } else {
      format!("{} media · filepack", self.fingerprint)
    }
  }
}
