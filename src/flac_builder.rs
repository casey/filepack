pub struct FlacBuilder {
  comments: Vec<(String, String)>,
  pictures: Vec<(u32, Vec<u8>)>,
  samples: u32,
  truncate: Option<usize>,
}

impl FlacBuilder {
  pub fn build(self) -> Vec<u8> {
    let mut streaminfo = Vec::new();
    streaminfo.extend_from_slice(&4096u16.to_be_bytes());
    streaminfo.extend_from_slice(&4096u16.to_be_bytes());
    streaminfo.extend_from_slice(&[0; 6]);
    streaminfo.extend_from_slice(&[0x0A, 0xC4, 0x42, 0xF0]);
    streaminfo.extend_from_slice(&self.samples.to_be_bytes());
    streaminfo.extend_from_slice(&[0; 16]);

    let mut blocks = vec![(0u8, streaminfo)];

    if !self.comments.is_empty() {
      let mut body = Vec::new();
      body.extend_from_slice(&0u32.to_le_bytes());
      body.extend_from_slice(&u32::try_from(self.comments.len()).unwrap().to_le_bytes());

      for (key, value) in self.comments {
        let comment = format!("{key}={value}");
        body.extend_from_slice(&u32::try_from(comment.len()).unwrap().to_le_bytes());
        body.extend_from_slice(comment.as_bytes());
      }

      blocks.push((4, body));
    }

    for (picture_type, data) in self.pictures {
      let mut body = Vec::new();
      body.extend_from_slice(&picture_type.to_be_bytes());
      body.extend_from_slice(&u32::try_from("image/png".len()).unwrap().to_be_bytes());
      body.extend_from_slice(b"image/png");
      body.extend_from_slice(&u32::try_from("bar".len()).unwrap().to_be_bytes());
      body.extend_from_slice(b"bar");
      body.extend_from_slice(&[0; 16]);
      body.extend_from_slice(&u32::try_from(data.len()).unwrap().to_be_bytes());
      body.extend_from_slice(&data);
      blocks.push((6, body));
    }

    let mut bytes = b"fLaC".to_vec();

    let last = blocks.len() - 1;
    for (i, (ty, body)) in blocks.into_iter().enumerate() {
      bytes.push(if i == last { ty | 0x80 } else { ty });
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
      pictures: Vec::new(),
      samples: 44100,
      truncate: None,
    }
  }

  #[must_use]
  pub fn picture(mut self, picture_type: u32, data: &[u8]) -> Self {
    self.pictures.push((picture_type, data.into()));
    self
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
