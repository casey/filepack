use super::*;

const MPEG1_BITRATES: [u64; 14] = [
  32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320,
];

const MPEG2_BITRATES: [u64; 14] = [8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160];

const SAMPLE_RATES: [u64; 3] = [44100, 48000, 32000];

struct Frame {
  channels: u64,
  metadata: bool,
  sample_rate: u64,
  samples: u64,
  size: usize,
}

pub(crate) struct Mp3Decoder<'a> {
  data: &'a [u8],
}

impl<'a> Mp3Decoder<'a> {
  pub(crate) fn decode(data: &'a [u8]) -> Result<AudioInfo, Mp3Error> {
    let decoder = Self { data };

    let mut offset = 0;
    let mut first = None;
    let mut samples = 0;

    while offset < decoder.data.len() {
      let frame = decoder.frame(offset)?;

      if let Some((channels, sample_rate)) = first {
        ensure! {
          frame.channels == channels,
          mp3_error::ChannelsMismatch {
            actual: frame.channels,
            expected: channels,
          },
        }

        ensure! {
          frame.sample_rate == sample_rate,
          mp3_error::SampleRateMismatch {
            actual: frame.sample_rate,
            expected: sample_rate,
          },
        }
      } else {
        first = Some((frame.channels, frame.sample_rate));
      }

      if !frame.metadata {
        samples += frame.samples;
      }

      offset += frame.size;
    }

    let (channels, sample_rate) = first.context(mp3_error::Empty)?;

    ensure!(samples > 0, mp3_error::Empty);

    Ok(AudioInfo {
      channels,
      sample_bits: None,
      sample_rate,
      samples,
    })
  }

  fn frame(&self, offset: usize) -> Result<Frame, Mp3Error> {
    let header = self
      .data
      .get(offset..offset + 4)
      .context(mp3_error::Truncated)?;

    ensure! {
      header[0] == 0xFF && header[1] & 0xE0 == 0xE0,
      mp3_error::Sync { offset },
    }

    let divisor = match (header[1] >> 3) & 0b11 {
      0 => 4,
      2 => 2,
      3 => 1,
      _ => return Err(mp3_error::Version.build()),
    };

    match (header[1] >> 1) & 0b11 {
      0 => return Err(mp3_error::LayerInvalid.build()),
      1 => {}
      bits => return Err(mp3_error::LayerUnsupported { layer: 4 - bits }.build()),
    }

    let index = header[2] >> 4;

    let bitrates = if divisor == 1 {
      MPEG1_BITRATES
    } else {
      MPEG2_BITRATES
    };

    ensure!((1..=14).contains(&index), mp3_error::Bitrate { index });

    let bitrate = bitrates[usize::from(index) - 1] * 1000;

    let sample_rate = match (header[2] >> 2) & 0b11 {
      3 => return Err(mp3_error::SampleRate.build()),
      index => SAMPLE_RATES[usize::from(index)] / divisor,
    };

    let padding = u64::from((header[2] >> 1) & 1);

    let channels = if header[3] >> 6 == 3 { 1 } else { 2 };

    let samples = if divisor == 1 { 1152 } else { 576 };

    let size = usize::try_from(samples / 8 * bitrate / sample_rate + padding).unwrap();

    ensure!(self.data.len() - offset >= size, mp3_error::Truncated);

    let side_info = match (divisor == 1, channels == 1) {
      (false, false) | (true, true) => 17,
      (false, true) => 9,
      (true, false) => 32,
    };

    let metadata = matches!(
      self
        .data
        .get(offset + 4 + side_info..offset + 8 + side_info),
      Some(b"Xing" | b"Info"),
    );

    Ok(Frame {
      channels,
      metadata,
      sample_rate,
      samples,
      size,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn decode() {
    #[track_caller]
    fn case(data: &[Vec<u8>], expected: Result<AudioInfo, Mp3Error>) {
      assert_eq!(Mp3Decoder::decode(&data.concat()), expected);
    }

    fn frame(header: [u8; 4], size: usize) -> Vec<u8> {
      let mut bytes = header.to_vec();
      bytes.resize(size, 0);
      bytes
    }

    fn info(channels: u64, sample_rate: u64, samples: u64) -> AudioInfo {
      AudioInfo {
        channels,
        sample_bits: None,
        sample_rate,
        samples,
      }
    }

    fn xing() -> Vec<u8> {
      let mut bytes = mp3_frame();
      bytes[36..40].copy_from_slice(b"Xing");
      bytes
    }

    let id3v1 = {
      let mut bytes = b"TAG".to_vec();
      bytes.resize(128, 0);
      bytes
    };

    case(&[mp3_frame(), mp3_frame()], Ok(info(2, 44100, 2304)));

    case(
      &[xing(), mp3_frame(), mp3_frame()],
      Ok(info(2, 44100, 2304)),
    );

    case(
      &[frame([0xFF, 0xFB, 0x92, 0x00], 418), mp3_frame()],
      Ok(info(2, 44100, 2304)),
    );

    case(
      &[frame([0xFF, 0xFB, 0x90, 0xC0], 417)],
      Ok(info(1, 44100, 1152)),
    );

    case(
      &[frame([0xFF, 0xF3, 0x90, 0x00], 261)],
      Ok(info(2, 22050, 576)),
    );

    case(
      &[frame([0xFF, 0xE3, 0x90, 0x00], 522)],
      Ok(info(2, 11025, 576)),
    );

    case(&[], Err(Mp3Error::Empty));

    case(&[xing()], Err(Mp3Error::Empty));

    case(&[mp3_frame(), id3v1], Err(Mp3Error::Sync { offset: 417 }));

    case(&[b"foobar".to_vec()], Err(Mp3Error::Sync { offset: 0 }));

    case(
      &[mp3_frame(), b"foobar".to_vec()],
      Err(Mp3Error::Sync { offset: 417 }),
    );

    case(&[mp3_frame()[..100].to_vec()], Err(Mp3Error::Truncated));

    case(&[mp3_frame(), b"bar".to_vec()], Err(Mp3Error::Truncated));

    case(
      &[frame([0xFF, 0xEB, 0x90, 0x00], 417)],
      Err(Mp3Error::Version),
    );

    case(
      &[frame([0xFF, 0xF9, 0x90, 0x00], 417)],
      Err(Mp3Error::LayerInvalid),
    );

    case(
      &[frame([0xFF, 0xFD, 0x90, 0x00], 417)],
      Err(Mp3Error::LayerUnsupported { layer: 2 }),
    );

    case(
      &[frame([0xFF, 0xFF, 0x90, 0x00], 417)],
      Err(Mp3Error::LayerUnsupported { layer: 1 }),
    );

    case(
      &[frame([0xFF, 0xFB, 0x00, 0x00], 417)],
      Err(Mp3Error::Bitrate { index: 0 }),
    );

    case(
      &[frame([0xFF, 0xFB, 0xF0, 0x00], 417)],
      Err(Mp3Error::Bitrate { index: 15 }),
    );

    case(
      &[frame([0xFF, 0xFB, 0x9C, 0x00], 417)],
      Err(Mp3Error::SampleRate),
    );

    case(
      &[mp3_frame(), frame([0xFF, 0xFB, 0x90, 0xC0], 417)],
      Err(Mp3Error::ChannelsMismatch {
        actual: 1,
        expected: 2,
      }),
    );

    case(
      &[mp3_frame(), frame([0xFF, 0xFB, 0x94, 0x00], 384)],
      Err(Mp3Error::SampleRateMismatch {
        actual: 48000,
        expected: 44100,
      }),
    );
  }
}
