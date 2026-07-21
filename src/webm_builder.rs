pub(crate) struct WebmBuilder {
  doc_type: String,
  tracks: Vec<Vec<u8>>,
}

impl WebmBuilder {
  #[must_use]
  pub(crate) fn audio_track(self, codec_id: &str) -> Self {
    self.track(2, codec_id, &[])
  }

  pub(crate) fn build(self) -> Vec<u8> {
    let header = [
      Self::string(&[0x42, 0x82], &self.doc_type),
      Self::unsigned(&[0x42, 0x87], 4),
      Self::unsigned(&[0x42, 0x85], 2),
    ]
    .concat();

    let info = [
      Self::string(&[0x4D, 0x80], "foo"),
      Self::string(&[0x57, 0x41], "bar"),
    ]
    .concat();

    let segment = [
      Self::element(&[0x15, 0x49, 0xA9, 0x66], &info),
      Self::element(&[0x16, 0x54, 0xAE, 0x6B], &self.tracks.concat()),
      Self::element(&[0x1F, 0x43, 0xB6, 0x75], &[]),
    ]
    .concat();

    [
      Self::element(&[0x1A, 0x45, 0xDF, 0xA3], &header),
      Self::element(&[0x18, 0x53, 0x80, 0x67], &segment),
    ]
    .concat()
  }

  #[must_use]
  pub(crate) fn doc_type(mut self, doc_type: &str) -> Self {
    self.doc_type = doc_type.into();
    self
  }

  fn element(id: &[u8], payload: &[u8]) -> Vec<u8> {
    let mut element = id.to_vec();
    element.push(0x01);
    element.extend_from_slice(&u64::try_from(payload.len()).unwrap().to_be_bytes()[1..]);
    element.extend_from_slice(payload);
    element
  }

  pub(crate) fn new() -> Self {
    Self {
      doc_type: "webm".into(),
      tracks: Vec::new(),
    }
  }

  fn string(id: &[u8], value: &str) -> Vec<u8> {
    Self::element(id, value.as_bytes())
  }

  #[must_use]
  pub(crate) fn track(mut self, ty: u64, codec_id: &str, settings: &[u8]) -> Self {
    let number = u64::try_from(self.tracks.len() + 1).unwrap();

    let entry = [
      Self::unsigned(&[0xD7], number),
      Self::unsigned(&[0x73, 0xC5], number),
      Self::unsigned(&[0x83], ty),
      Self::string(&[0x86], codec_id),
      settings.to_vec(),
    ]
    .concat();

    self.tracks.push(Self::element(&[0xAE], &entry));

    self
  }

  fn unsigned(id: &[u8], value: u64) -> Vec<u8> {
    Self::element(id, &value.to_be_bytes())
  }

  pub(crate) fn video_settings(width: u64, height: u64) -> Vec<u8> {
    Self::element(
      &[0xE0],
      &[
        Self::unsigned(&[0xB0], width),
        Self::unsigned(&[0xBA], height),
      ]
      .concat(),
    )
  }

  #[must_use]
  pub(crate) fn video_track(self, width: u64, height: u64) -> Self {
    let settings = Self::video_settings(width, height);
    self.track(1, "V_VP9", &settings)
  }
}
