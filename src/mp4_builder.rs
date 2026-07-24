pub struct Mp4Builder {
  duration: u32,
  frame_count: u32,
  sample_size: u32,
  sample_sizes: Vec<u32>,
  timescale: u32,
  tracks: Vec<Vec<u8>>,
}

impl Mp4Builder {
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

    let mut mvhd = vec![0; 12];
    mvhd.extend_from_slice(&self.timescale.to_be_bytes());
    mvhd.extend_from_slice(&self.duration.to_be_bytes());
    mvhd.extend_from_slice(&0x0001_0000u32.to_be_bytes());
    mvhd.extend_from_slice(&[0; 76]);

    let moov = [Self::atom(*b"mvhd", &mvhd), self.tracks.concat()].concat();

    [Self::atom(*b"ftyp", &ftyp), Self::atom(*b"moov", &moov)].concat()
  }

  #[must_use]
  pub fn duration(mut self, duration: u32) -> Self {
    self.duration = duration;
    self
  }

  #[must_use]
  pub fn frame_count(mut self, frame_count: u32) -> Self {
    self.frame_count = frame_count;
    self
  }

  pub fn new() -> Self {
    Self {
      duration: 0,
      frame_count: 0,
      sample_size: 1,
      sample_sizes: Vec::new(),
      timescale: 1000,
      tracks: Vec::new(),
    }
  }

  #[must_use]
  pub fn sample_size(mut self, sample_size: u32) -> Self {
    self.sample_size = sample_size;
    self
  }

  #[must_use]
  pub fn sample_sizes(mut self, sample_sizes: &[u32]) -> Self {
    self.frame_count = sample_sizes.len().try_into().unwrap();
    self.sample_sizes = sample_sizes.into();
    self
  }

  #[must_use]
  pub fn timescale(mut self, timescale: u32) -> Self {
    self.timescale = timescale;
    self
  }

  #[must_use]
  pub fn track(mut self, handler: [u8; 4], descriptions: &[Vec<u8>]) -> Self {
    let mut tkhd = vec![0; 12];
    tkhd.extend_from_slice(&u32::try_from(self.tracks.len() + 1).unwrap().to_be_bytes());
    tkhd.extend_from_slice(&[0; 68]);

    let mut mdhd = vec![0; 12];
    mdhd.extend_from_slice(&1000u32.to_be_bytes());
    mdhd.extend_from_slice(&self.frame_count.to_be_bytes());
    mdhd.extend_from_slice(&[0; 4]);

    let mut hdlr = vec![0; 8];
    hdlr.extend_from_slice(&handler);
    hdlr.extend_from_slice(&[0; 12]);
    hdlr.push(0);

    let dinf = Self::atom(*b"dinf", &Self::atom(*b"dref", &[0; 8]));

    let mut stsd = vec![0, 0, 0, 0];
    stsd.extend_from_slice(&u32::try_from(descriptions.len()).unwrap().to_be_bytes());
    stsd.extend_from_slice(&descriptions.concat());

    let stbl = if self.frame_count > 0 {
      let mut stts = vec![0; 4];
      stts.extend_from_slice(&1u32.to_be_bytes());
      stts.extend_from_slice(&self.frame_count.to_be_bytes());
      stts.extend_from_slice(&1u32.to_be_bytes());

      let mut stsc = vec![0; 4];
      stsc.extend_from_slice(&1u32.to_be_bytes());
      stsc.extend_from_slice(&1u32.to_be_bytes());
      stsc.extend_from_slice(&self.frame_count.to_be_bytes());
      stsc.extend_from_slice(&1u32.to_be_bytes());

      let mut stsz = vec![0; 4];

      if self.sample_sizes.is_empty() {
        stsz.extend_from_slice(&self.sample_size.to_be_bytes());
        stsz.extend_from_slice(&self.frame_count.to_be_bytes());
      } else {
        stsz.extend_from_slice(&0u32.to_be_bytes());
        stsz.extend_from_slice(&self.frame_count.to_be_bytes());
        for sample_size in &self.sample_sizes {
          stsz.extend_from_slice(&sample_size.to_be_bytes());
        }
      }

      let mut stco = vec![0; 4];
      stco.extend_from_slice(&1u32.to_be_bytes());
      stco.extend_from_slice(&0u32.to_be_bytes());

      [
        Self::atom(*b"stsd", &stsd),
        Self::atom(*b"stts", &stts),
        Self::atom(*b"stsc", &stsc),
        Self::atom(*b"stsz", &stsz),
        Self::atom(*b"stco", &stco),
      ]
      .concat()
    } else {
      [
        Self::atom(*b"stsd", &stsd),
        Self::atom(*b"stts", &[0; 8]),
        Self::atom(*b"stsc", &[0; 8]),
        Self::atom(*b"stsz", &[0; 12]),
        Self::atom(*b"stco", &[0; 8]),
      ]
      .concat()
    };

    let minf = [dinf, Self::atom(*b"stbl", &stbl)].concat();

    let mdia = [
      Self::atom(*b"mdhd", &mdhd),
      Self::atom(*b"hdlr", &hdlr),
      Self::atom(*b"minf", &minf),
    ]
    .concat();

    let trak = [Self::atom(*b"tkhd", &tkhd), Self::atom(*b"mdia", &mdia)].concat();

    self.tracks.push(Self::atom(*b"trak", &trak));

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
    payload.extend_from_slice(&Self::atom(config, &[1, 0, 0, 0, 0xff, 0xe0, 0]));

    Self::atom(entry, &payload)
  }

  #[must_use]
  pub fn video_track(self, width: u16, height: u16) -> Self {
    let entry = Self::video_entry(*b"avc1", *b"avcC", width, height);
    self.track(*b"vide", &[entry])
  }
}
