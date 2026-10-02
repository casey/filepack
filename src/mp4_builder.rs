pub struct Mp4Builder {
  avcc_profile: u8,
  duration: u32,
  esds_long_lengths: bool,
  frame_count: u32,
  matrix: [i32; 9],
  media_timescale: Option<u32>,
  meta_in_moov: bool,
  mp4a_version: u16,
  sample_size: u32,
  sample_sizes: Vec<u32>,
  sps: Vec<u8>,
  tags: Vec<Vec<u8>>,
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

  fn audio_entry(&self, object_type: u8) -> Vec<u8> {
    let length = |length: usize| -> Vec<u8> {
      let length = u8::try_from(length).unwrap();
      if self.esds_long_lengths {
        vec![0x80, 0x80, 0x80, length]
      } else {
        vec![length]
      }
    };

    let mut descriptor = vec![0x04];
    descriptor.extend_from_slice(&length(13));
    descriptor.push(object_type);
    descriptor.extend_from_slice(&[0; 12]);

    let mut es = vec![0x03];
    es.extend_from_slice(&length(descriptor.len() + 3));
    es.extend_from_slice(&[0, 1, 0]);
    es.extend_from_slice(&descriptor);

    let mut esds = vec![0, 0, 0, 0];
    esds.extend_from_slice(&es);

    let mut payload = Vec::new();
    payload.extend_from_slice(&[0; 6]);
    payload.extend_from_slice(&[0, 1]);
    payload.extend_from_slice(&self.mp4a_version.to_be_bytes());
    payload.extend_from_slice(&[0; 6]);
    payload.extend_from_slice(&2u16.to_be_bytes());
    payload.extend_from_slice(&16u16.to_be_bytes());
    payload.extend_from_slice(&[0; 4]);
    payload.extend_from_slice(&(44100u32 << 16).to_be_bytes());
    if self.mp4a_version == 1 {
      payload.extend_from_slice(&[0; 16]);
    }
    payload.extend_from_slice(&Self::atom(*b"esds", &esds));

    Self::atom(*b"mp4a", &payload)
  }

  #[must_use]
  pub fn audio_track(self, object_type: u8) -> Self {
    let entry = self.audio_entry(object_type);
    let timescale = self.media_timescale.unwrap_or(44100);
    self.track(*b"soun", timescale, &[entry])
  }

  #[cfg(test)]
  #[must_use]
  pub(crate) fn avcc_profile(mut self, avcc_profile: u8) -> Self {
    self.avcc_profile = avcc_profile;
    self
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

    let udta = if self.tags.is_empty() {
      Vec::new()
    } else {
      let mut hdlr = vec![0; 8];
      hdlr.extend_from_slice(b"mdir");
      hdlr.extend_from_slice(&[0; 12]);
      hdlr.push(0);

      let ilst = Self::atom(*b"ilst", &self.tags.concat());

      let meta = [vec![0; 4], Self::atom(*b"hdlr", &hdlr), ilst].concat();

      if self.meta_in_moov {
        Self::atom(*b"meta", &meta)
      } else {
        Self::atom(*b"udta", &Self::atom(*b"meta", &meta))
      }
    };

    let moov = [Self::atom(*b"mvhd", &mvhd), self.tracks.concat(), udta].concat();

    [Self::atom(*b"ftyp", &ftyp), Self::atom(*b"moov", &moov)].concat()
  }

  #[cfg(test)]
  #[must_use]
  pub(crate) fn data(mut self, fourcc: [u8; 4], data_type: u32, payload: &[u8]) -> Self {
    let mut data = data_type.to_be_bytes().to_vec();
    data.extend_from_slice(&[0; 4]);
    data.extend_from_slice(payload);
    self
      .tags
      .push(Self::atom(fourcc, &Self::atom(*b"data", &data)));
    self
  }

  #[must_use]
  pub fn duration(mut self, duration: u32) -> Self {
    self.duration = duration;
    self
  }

  #[cfg(test)]
  #[must_use]
  pub(crate) fn esds_long_lengths(mut self) -> Self {
    self.esds_long_lengths = true;
    self
  }

  #[must_use]
  pub fn frame_count(mut self, frame_count: u32) -> Self {
    self.frame_count = frame_count;
    self
  }

  #[cfg(test)]
  #[must_use]
  pub(crate) fn matrix(mut self, matrix: [i32; 9]) -> Self {
    self.matrix = matrix;
    self
  }

  #[cfg(test)]
  #[must_use]
  pub(crate) fn media_timescale(mut self, media_timescale: u32) -> Self {
    self.media_timescale = Some(media_timescale);
    self
  }

  #[cfg(test)]
  #[must_use]
  pub(crate) fn meta_in_moov(mut self) -> Self {
    self.meta_in_moov = true;
    self
  }

  #[cfg(test)]
  #[must_use]
  pub(crate) fn mp4a_version(mut self, mp4a_version: u16) -> Self {
    self.mp4a_version = mp4a_version;
    self
  }

  #[must_use]
  pub fn name(self, name: impl AsRef<[u8]>) -> Self {
    self.tag(*b"\xa9nam", name)
  }

  pub fn new() -> Self {
    Self {
      avcc_profile: 0,
      duration: 0,
      esds_long_lengths: false,
      frame_count: 0,
      matrix: [0x0001_0000, 0, 0, 0, 0x0001_0000, 0, 0, 0, 0x4000_0000],
      media_timescale: None,
      meta_in_moov: false,
      mp4a_version: 0,
      sample_size: 1,
      sample_sizes: Vec::new(),
      sps: Vec::new(),
      tags: Vec::new(),
      timescale: 1000,
      tracks: Vec::new(),
    }
  }

  #[must_use]
  pub fn pair_tag(mut self, fourcc: [u8; 4], number: u16, total: u16) -> Self {
    let mut data = 0u32.to_be_bytes().to_vec();
    data.extend_from_slice(&[0; 4]);
    data.extend_from_slice(&[0; 2]);
    data.extend_from_slice(&number.to_be_bytes());
    data.extend_from_slice(&total.to_be_bytes());
    data.extend_from_slice(&[0; 2]);
    self
      .tags
      .push(Self::atom(fourcc, &Self::atom(*b"data", &data)));
    self
  }

  #[must_use]
  pub fn picture(mut self, data_type: u32, picture: &[u8]) -> Self {
    let mut data = data_type.to_be_bytes().to_vec();
    data.extend_from_slice(&[0; 4]);
    data.extend_from_slice(picture);
    self
      .tags
      .push(Self::atom(*b"covr", &Self::atom(*b"data", &data)));
    self
  }

  #[cfg(test)]
  #[must_use]
  pub(crate) fn sample_size(mut self, sample_size: u32) -> Self {
    self.sample_size = sample_size;
    self
  }

  #[cfg(test)]
  #[must_use]
  pub(crate) fn sample_sizes(mut self, sample_sizes: &[u32]) -> Self {
    self.frame_count = sample_sizes.len().try_into().unwrap();
    self.sample_sizes = sample_sizes.into();
    self
  }

  #[cfg(test)]
  #[must_use]
  pub(crate) fn sps(mut self, sps: &[u8]) -> Self {
    self.sps = sps.into();
    self
  }

  #[must_use]
  pub fn tag(mut self, fourcc: [u8; 4], value: impl AsRef<[u8]>) -> Self {
    let mut data = 1u32.to_be_bytes().to_vec();
    data.extend_from_slice(&[0; 4]);
    data.extend_from_slice(value.as_ref());
    self
      .tags
      .push(Self::atom(fourcc, &Self::atom(*b"data", &data)));
    self
  }

  #[cfg(test)]
  #[must_use]
  pub(crate) fn timescale(mut self, timescale: u32) -> Self {
    self.timescale = timescale;
    self
  }

  #[must_use]
  pub(crate) fn track(
    mut self,
    handler: [u8; 4],
    timescale: u32,
    descriptions: &[Vec<u8>],
  ) -> Self {
    let mut tkhd = vec![0; 12];
    tkhd.extend_from_slice(&u32::try_from(self.tracks.len() + 1).unwrap().to_be_bytes());
    tkhd.extend_from_slice(&[0; 24]);
    for value in self.matrix {
      tkhd.extend_from_slice(&value.to_be_bytes());
    }
    tkhd.extend_from_slice(&[0; 8]);

    let mut mdhd = vec![0; 12];
    mdhd.extend_from_slice(&timescale.to_be_bytes());
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

  pub(crate) fn video_entry(
    entry: [u8; 4],
    config: [u8; 4],
    config_payload: &[u8],
    width: u16,
    height: u16,
  ) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&[0; 6]);
    payload.extend_from_slice(&[0, 1]);
    payload.extend_from_slice(&[0; 16]);
    payload.extend_from_slice(&width.to_be_bytes());
    payload.extend_from_slice(&height.to_be_bytes());
    payload.extend_from_slice(&[0; 50]);
    payload.extend_from_slice(&Self::atom(config, config_payload));

    Self::atom(entry, &payload)
  }

  #[must_use]
  pub fn video_track(self, width: u16, height: u16) -> Self {
    let avcc = if self.sps.is_empty() {
      vec![1, self.avcc_profile, 0, 0, 0xff, 0xe0, 0]
    } else {
      let mut avcc = vec![1, self.sps[1], self.sps[2], self.sps[3], 0xff, 0xe1];
      avcc.extend_from_slice(&u16::try_from(self.sps.len()).unwrap().to_be_bytes());
      avcc.extend_from_slice(&self.sps);
      avcc.push(0);
      avcc
    };

    let entry = Self::video_entry(*b"avc1", *b"avcC", &avcc, width, height);
    self.track(*b"vide", 1000, &[entry])
  }
}
