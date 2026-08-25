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
  xmp: Option<Vec<u8>>,
}

impl JpegBuilder {
  fn app1(header: &[u8], payload: &[u8]) -> Vec<u8> {
    let mut segment = vec![0xFF, 0xE1];
    segment.extend_from_slice(
      &u16::try_from(header.len() + payload.len() + 2)
        .unwrap()
        .to_be_bytes(),
    );
    segment.extend_from_slice(header);
    segment.extend_from_slice(payload);
    segment
  }

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

    let mut segments = Vec::new();

    if let Some(exif) = self.exif {
      segments.push(Self::app1(b"Exif\0\0", &exif));
    }

    if let Some(xmp) = self.xmp {
      segments.push(Self::app1(b"http://ns.adobe.com/xap/1.0/\0", &xmp));
    }

    [&bytes[..2], &segments.concat(), &bytes[2..]].concat()
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
      xmp: None,
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

  #[must_use]
  pub(crate) fn xmp(mut self, xmp: &[u8]) -> Self {
    self.xmp = Some(xmp.into());
    self
  }
}
