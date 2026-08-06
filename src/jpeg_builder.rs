use {
  super::*,
  ::image::{DynamicImage, ImageFormat},
};

pub(crate) struct JpegBuilder {
  exif: Option<Vec<u8>>,
  grayscale: bool,
  height: u32,
  sampling: Option<u8>,
  width: u32,
}

impl JpegBuilder {
  pub(crate) fn build(self) -> Vec<u8> {
    let image = if self.grayscale {
      DynamicImage::new_luma8(self.width, self.height)
    } else {
      DynamicImage::new_rgb8(self.width, self.height)
    };

    let mut buffer = io::Cursor::new(Vec::new());

    image.write_to(&mut buffer, ImageFormat::Jpeg).unwrap();

    let mut bytes = buffer.into_inner();

    if let Some(sampling) = self.sampling {
      let sof = bytes.windows(2).position(|w| w == [0xFF, 0xC0]).unwrap();
      bytes[sof + 11] = sampling;
    }

    if let Some(exif) = self.exif {
      let mut app1 = b"Exif\0\0".to_vec();
      app1.extend_from_slice(&exif);

      let mut spliced = bytes[..2].to_vec();
      spliced.extend_from_slice(&[0xFF, 0xE1]);
      spliced.extend_from_slice(&u16::try_from(app1.len() + 2).unwrap().to_be_bytes());
      spliced.extend_from_slice(&app1);
      spliced.extend_from_slice(&bytes[2..]);
      spliced
    } else {
      bytes
    }
  }

  #[must_use]
  pub(crate) fn exif(mut self, exif: &[u8]) -> Self {
    self.exif = Some(exif.into());
    self
  }

  #[must_use]
  pub(crate) fn grayscale(mut self) -> Self {
    self.grayscale = true;
    self
  }

  #[must_use]
  pub(crate) fn height(mut self, height: u32) -> Self {
    self.height = height;
    self
  }

  pub(crate) fn new() -> Self {
    Self {
      exif: None,
      grayscale: false,
      height: 1,
      sampling: None,
      width: 1,
    }
  }

  #[must_use]
  pub(crate) fn sampling(mut self, sampling: u8) -> Self {
    self.sampling = Some(sampling);
    self
  }

  #[must_use]
  pub(crate) fn width(mut self, width: u32) -> Self {
    self.width = width;
    self
  }
}
