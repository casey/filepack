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
  fn title(&self) -> String {
    if let Some(title) = self.title() {
      format!("{title} media · Filepack")
    } else {
      format!("{} media · Filepack", self.fingerprint)
    }
  }
}
