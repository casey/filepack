use super::*;

#[derive(Boilerplate)]
pub(crate) struct VideoHtml {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) video: usize,
}

impl Page for VideoHtml {
  fn title(&self) -> String {
    format!("video {} · filepack", self.video)
  }
}
