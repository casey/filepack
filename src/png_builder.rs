use png::{BitDepth, chunk::eXIf};

pub(crate) struct PngBuilder {
  color: png::ColorType,
  depth: BitDepth,
  exif: Option<Vec<u8>>,
  height: u32,
  trns: Option<Vec<u8>>,
  width: u32,
}

impl PngBuilder {
  pub(crate) fn build(self) -> Vec<u8> {
    let mut buffer = Vec::new();

    let mut encoder = png::Encoder::new(&mut buffer, self.width, self.height);
    encoder.set_color(self.color);
    encoder.set_depth(self.depth);

    if self.color == png::ColorType::Indexed {
      encoder.set_palette(vec![0; 3]);
    }

    if let Some(trns) = self.trns {
      encoder.set_trns(trns);
    }

    let mut writer = encoder.write_header().unwrap();

    if let Some(exif) = self.exif {
      writer.write_chunk(eXIf, &exif).unwrap();
    }

    let samples = u32::try_from(self.color.samples()).unwrap();
    let row = (self.width * samples * u32::from(self.depth as u8)).div_ceil(8);

    writer
      .write_image_data(&vec![0; usize::try_from(row * self.height).unwrap()])
      .unwrap();
    writer.finish().unwrap();

    buffer
  }

  #[must_use]
  pub(crate) fn color(mut self, color: png::ColorType) -> Self {
    self.color = color;
    self
  }

  #[must_use]
  pub(crate) fn depth(mut self, depth: BitDepth) -> Self {
    self.depth = depth;
    self
  }

  #[must_use]
  pub(crate) fn exif(mut self, exif: &[u8]) -> Self {
    self.exif = Some(exif.into());
    self
  }

  #[must_use]
  pub(crate) fn height(mut self, height: u32) -> Self {
    self.height = height;
    self
  }

  pub(crate) fn new() -> Self {
    Self {
      color: png::ColorType::Rgb,
      depth: BitDepth::Eight,
      exif: None,
      height: 1,
      trns: None,
      width: 1,
    }
  }

  #[must_use]
  pub(crate) fn trns(mut self, trns: &[u8]) -> Self {
    self.trns = Some(trns.into());
    self
  }

  #[must_use]
  pub(crate) fn width(mut self, width: u32) -> Self {
    self.width = width;
    self
  }
}
