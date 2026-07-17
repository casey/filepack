pub struct VideoBuilder {
  traks: Vec<Vec<u8>>,
}

impl VideoBuilder {
  fn atom(fourcc: [u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut atom = Vec::new();
    atom.extend_from_slice(&u32::try_from(payload.len() + 8).unwrap().to_be_bytes());
    atom.extend_from_slice(&fourcc);
    atom.extend_from_slice(payload);
    atom
  }

  pub fn audio_entry(object_type: u8) -> Vec<u8> {
    let mut descriptor = vec![0x04, 13, object_type];
    descriptor.extend_from_slice(&[0; 12]);

    let mut es = vec![0x03, u8::try_from(descriptor.len() + 3).unwrap(), 0, 1, 0];
    es.extend_from_slice(&descriptor);

    let mut esds = vec![0, 0, 0, 0];
    esds.extend_from_slice(&es);

    let mut payload = Vec::new();
    payload.extend_from_slice(&[0; 6]);
    payload.extend_from_slice(&[0, 1]);
    payload.extend_from_slice(&[0; 8]);
    payload.extend_from_slice(&2u16.to_be_bytes());
    payload.extend_from_slice(&16u16.to_be_bytes());
    payload.extend_from_slice(&[0; 4]);
    payload.extend_from_slice(&(44100u32 << 16).to_be_bytes());
    payload.extend_from_slice(&Self::atom(*b"esds", &esds));

    Self::atom(*b"mp4a", &payload)
  }

  #[must_use]
  pub fn audio_track(self, object_type: u8) -> Self {
    let entry = Self::audio_entry(object_type);
    self.track(*b"soun", &[entry])
  }

  pub fn build(self) -> Vec<u8> {
    let mut ftyp = Vec::new();
    ftyp.extend_from_slice(b"isom");
    ftyp.extend_from_slice(&[0; 4]);
    ftyp.extend_from_slice(b"isom");

    [
      Self::atom(*b"ftyp", &ftyp),
      Self::atom(*b"moov", &self.traks.concat()),
    ]
    .concat()
  }

  pub fn new() -> Self {
    Self { traks: Vec::new() }
  }

  #[must_use]
  pub fn track(mut self, handler: [u8; 4], descriptions: &[Vec<u8>]) -> Self {
    let mut hdlr = vec![0; 8];
    hdlr.extend_from_slice(&handler);
    hdlr.extend_from_slice(&[0; 12]);
    hdlr.push(0);

    let mut stsd = vec![0, 0, 0, 0];
    stsd.extend_from_slice(&u32::try_from(descriptions.len()).unwrap().to_be_bytes());
    stsd.extend_from_slice(&descriptions.concat());

    let stbl = Self::atom(*b"stbl", &Self::atom(*b"stsd", &stsd));
    let minf = Self::atom(*b"minf", &stbl);
    let mdia = [Self::atom(*b"hdlr", &hdlr), minf].concat();

    self
      .traks
      .push(Self::atom(*b"trak", &Self::atom(*b"mdia", &mdia)));

    self
  }

  pub fn video_entry(entry: [u8; 4], config: [u8; 4], width: u16, height: u16) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&[0; 6]);
    payload.extend_from_slice(&[0, 1]);
    payload.extend_from_slice(&[0; 16]);
    payload.extend_from_slice(&width.to_be_bytes());
    payload.extend_from_slice(&height.to_be_bytes());
    payload.extend_from_slice(&[0; 50]);
    payload.extend_from_slice(&Self::atom(config, &[]));

    Self::atom(entry, &payload)
  }

  #[must_use]
  pub fn video_track(self, width: u16, height: u16) -> Self {
    let entry = Self::video_entry(*b"s263", *b"d263", width, height);
    self.track(*b"vide", &[entry])
  }
}
