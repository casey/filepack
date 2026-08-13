pub(crate) struct Mp3Builder {
  frames: Vec<Vec<u8>>,
  id3v1: bool,
  id3v2: Option<Vec<(String, String)>>,
  trailing: Vec<u8>,
  truncate: Option<usize>,
}

impl Mp3Builder {
  pub(crate) fn build(self) -> Vec<u8> {
    let mut bytes = Vec::new();

    if let Some(tags) = self.id3v2 {
      let mut body = Vec::new();

      for (id, value) in tags {
        body.extend_from_slice(id.as_bytes());
        body.extend_from_slice(&Self::syncsafe(value.len() + 1));
        body.extend_from_slice(&[0; 2]);
        body.push(3);
        body.extend_from_slice(value.as_bytes());
      }

      bytes.extend_from_slice(b"ID3");
      bytes.extend_from_slice(&[4, 0, 0]);
      bytes.extend_from_slice(&Self::syncsafe(body.len()));
      bytes.extend(body);
    }

    bytes.extend(self.frames.concat());

    if self.id3v1 {
      let mut tag = b"TAG".to_vec();
      tag.resize(128, 0);
      bytes.extend(tag);
    }

    bytes.extend(self.trailing);

    if let Some(len) = self.truncate {
      bytes.truncate(len);
    }

    bytes
  }

  #[must_use]
  pub(crate) fn frame(mut self, header: [u8; 4], size: usize) -> Self {
    let mut bytes = header.to_vec();
    bytes.resize(size, 0);
    self.frames.push(bytes);
    self
  }

  #[must_use]
  pub(crate) fn frames(mut self, count: u32) -> Self {
    for _ in 0..count {
      self.frames.push(Self::standard());
    }
    self
  }

  #[must_use]
  pub(crate) fn id3v1(mut self) -> Self {
    self.id3v1 = true;
    self
  }

  #[must_use]
  pub(crate) fn id3v2(mut self) -> Self {
    self.id3v2.get_or_insert_default();
    self
  }

  pub(crate) fn new() -> Self {
    Self {
      frames: Vec::new(),
      id3v1: false,
      id3v2: None,
      trailing: Vec::new(),
      truncate: None,
    }
  }

  fn standard() -> Vec<u8> {
    let mut bytes = vec![0xFF, 0xFB, 0x90, 0x00];
    bytes.resize(417, 0);
    bytes
  }

  fn syncsafe(n: usize) -> [u8; 4] {
    let n = u32::try_from(n).unwrap();
    [
      u8::try_from((n >> 21) & 0x7F).unwrap(),
      u8::try_from((n >> 14) & 0x7F).unwrap(),
      u8::try_from((n >> 7) & 0x7F).unwrap(),
      u8::try_from(n & 0x7F).unwrap(),
    ]
  }

  #[must_use]
  pub(crate) fn tag(mut self, id: &str, value: &str) -> Self {
    self
      .id3v2
      .get_or_insert_default()
      .push((id.into(), value.into()));
    self
  }

  #[must_use]
  pub(crate) fn trailing(mut self, trailing: &[u8]) -> Self {
    self.trailing.extend_from_slice(trailing);
    self
  }

  #[must_use]
  pub(crate) fn truncate(mut self, len: usize) -> Self {
    self.truncate = Some(len);
    self
  }

  #[must_use]
  pub(crate) fn xing(mut self) -> Self {
    let mut bytes = Self::standard();
    bytes[36..40].copy_from_slice(b"Xing");
    self.frames.push(bytes);
    self
  }
}
