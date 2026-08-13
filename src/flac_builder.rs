pub struct FlacBuilder {
  comments: Vec<(String, String)>,
  samples: u32,
  truncate: Option<usize>,
}

impl FlacBuilder {
  pub fn build(self) -> Vec<u8> {
    let mut bytes = b"fLaC".to_vec();

    bytes.push(if self.comments.is_empty() { 0x80 } else { 0x00 });
    bytes.extend_from_slice(&34u32.to_be_bytes()[1..]);
    bytes.extend_from_slice(&4096u16.to_be_bytes());
    bytes.extend_from_slice(&4096u16.to_be_bytes());
    bytes.extend_from_slice(&[0; 6]);
    bytes.extend_from_slice(&[0x0A, 0xC4, 0x42, 0xF0]);
    bytes.extend_from_slice(&self.samples.to_be_bytes());
    bytes.extend_from_slice(&[0; 16]);

    if !self.comments.is_empty() {
      let mut body = Vec::new();
      body.extend_from_slice(&0u32.to_le_bytes());
      body.extend_from_slice(&u32::try_from(self.comments.len()).unwrap().to_le_bytes());

      for (key, value) in self.comments {
        let comment = format!("{key}={value}");
        body.extend_from_slice(&u32::try_from(comment.len()).unwrap().to_le_bytes());
        body.extend_from_slice(comment.as_bytes());
      }

      bytes.push(0x84);
      bytes.extend_from_slice(&u32::try_from(body.len()).unwrap().to_be_bytes()[1..]);
      bytes.extend(body);
    }

    bytes.extend_from_slice(&[0; 1024]);

    if let Some(len) = self.truncate {
      bytes.truncate(len);
    }

    bytes
  }

  pub fn new() -> Self {
    Self {
      comments: Vec::new(),
      samples: 44100,
      truncate: None,
    }
  }

  #[must_use]
  pub fn samples(mut self, samples: u32) -> Self {
    self.samples = samples;
    self
  }

  #[must_use]
  pub fn tag(mut self, key: &str, value: &str) -> Self {
    self.comments.push((key.into(), value.into()));
    self
  }

  #[must_use]
  pub fn truncate(mut self, len: usize) -> Self {
    self.truncate = Some(len);
    self
  }
}
