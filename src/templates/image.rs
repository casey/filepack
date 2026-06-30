use super::*;

#[derive(Boilerplate)]
pub(crate) struct ImageHtml {
  pub(crate) fingerprint: Fingerprint,
  pub(crate) image: usize,
  pub(crate) metadata: Metadata,
}

impl ImageHtml {
  fn image(&self) -> &Image {
    let Media::Image { images } = self.metadata.media.as_ref().unwrap() else {
      unreachable!();
    };

    &images[self.image]
  }
}

impl Page for ImageHtml {
  fn stylesheet(&self) -> Option<&str> {
    Some("/static/image.css")
  }

  fn title(&self) -> String {
    format!("image {} · filepack", self.image)
  }
}
