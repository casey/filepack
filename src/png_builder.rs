use png::{BitDepth, chunk};

pub struct PngBuilder {
  color: png::ColorType,
  depth: BitDepth,
  exif: Option<Vec<u8>>,
  height: u32,
  itxt: Vec<(String, String)>,
  text: Vec<(String, String)>,
  trailing_text: Vec<(String, String)>,
  trns: Option<Vec<u8>>,
  width: u32,
  ztxt: Vec<(String, String)>,
}

impl PngBuilder {
  pub fn build(self) -> Vec<u8> {
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

    for (keyword, text) in self.text {
      encoder.add_text_chunk(keyword, text).unwrap();
    }

    for (keyword, text) in self.itxt {
      encoder.add_itxt_chunk(keyword, text).unwrap();
    }

    for (keyword, text) in self.ztxt {
      encoder.add_ztxt_chunk(keyword, text).unwrap();
    }

    let mut writer = encoder.write_header().unwrap();

    if let Some(exif) = self.exif {
      writer.write_chunk(chunk::eXIf, &exif).unwrap();
    }

    let samples = u32::try_from(self.color.samples()).unwrap();
    let row = (self.width * samples * u32::from(self.depth as u8)).div_ceil(8);

    writer
      .write_image_data(&vec![0; usize::try_from(row * self.height).unwrap()])
      .unwrap();

    for (keyword, text) in self.trailing_text {
      writer
        .write_chunk(
          chunk::tEXt,
          &[keyword.as_bytes(), b"\0", text.as_bytes()].concat(),
        )
        .unwrap();
    }

    writer.finish().unwrap();

    buffer
  }

  #[must_use]
  pub fn color(mut self, color: png::ColorType) -> Self {
    self.color = color;
    self
  }

  #[must_use]
  pub fn depth(mut self, depth: BitDepth) -> Self {
    self.depth = depth;
    self
  }

  #[must_use]
  pub fn exif(mut self, exif: &[u8]) -> Self {
    self.exif = Some(exif.into());
    self
  }

  #[must_use]
  pub fn height(mut self, height: u32) -> Self {
    self.height = height;
    self
  }

  #[must_use]
  pub fn itxt(mut self, keyword: &str, text: &str) -> Self {
    self.itxt.push((keyword.into(), text.into()));
    self
  }

  pub fn new() -> Self {
    Self {
      color: png::ColorType::Rgb,
      depth: BitDepth::Eight,
      exif: None,
      height: 1,
      itxt: Vec::new(),
      text: Vec::new(),
      trailing_text: Vec::new(),
      trns: None,
      width: 1,
      ztxt: Vec::new(),
    }
  }

  #[must_use]
  pub fn text(mut self, keyword: &str, text: &str) -> Self {
    self.text.push((keyword.into(), text.into()));
    self
  }

  #[must_use]
  pub fn trailing_text(mut self, keyword: &str, text: &str) -> Self {
    self.trailing_text.push((keyword.into(), text.into()));
    self
  }

  #[must_use]
  pub fn trns(mut self, trns: &[u8]) -> Self {
    self.trns = Some(trns.into());
    self
  }

  #[must_use]
  pub fn width(mut self, width: u32) -> Self {
    self.width = width;
    self
  }

  #[must_use]
  pub fn ztxt(mut self, keyword: &str, text: &str) -> Self {
    self.ztxt.push((keyword.into(), text.into()));
    self
  }
}
