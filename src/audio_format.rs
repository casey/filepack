use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct AudioFormat {
  pub(crate) channels: u64,
  pub(crate) sample_bits: u64,
  pub(crate) sample_rate: u64,
  pub(crate) ty: AudioType,
}

impl Display for AudioFormat {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self.ty {
      AudioType::Flac => write!(f, "FLAC")?,
    }

    write!(f, " · {}-bit", self.sample_bits)?;

    let khz = self.sample_rate / 1000;
    let frac = format!("{:03}", self.sample_rate % 1000);
    let frac = frac.trim_end_matches('0');

    if frac.is_empty() {
      write!(f, " {khz} kHz")?;
    } else {
      write!(f, " {khz}.{frac} kHz")?;
    }

    match self.channels {
      1 => write!(f, " mono")?,
      2 => write!(f, " stereo")?,
      6 => write!(f, " 5.1")?,
      8 => write!(f, " 7.1")?,
      channels => write!(f, " {channels} channels")?,
    }

    match self.ty {
      AudioType::Flac => write!(f, " · lossless")?,
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    #[track_caller]
    fn case(channels: u64, sample_bits: u64, sample_rate: u64, expected: &str) {
      assert_eq!(
        AudioFormat {
          channels,
          sample_bits,
          sample_rate,
          ty: AudioType::Flac,
        }
        .to_string(),
        expected,
      );
    }

    case(2, 16, 44100, "FLAC · 16-bit 44.1 kHz stereo · lossless");
    case(2, 24, 96000, "FLAC · 24-bit 96 kHz stereo · lossless");
    case(1, 16, 22050, "FLAC · 16-bit 22.05 kHz mono · lossless");
    case(6, 24, 48000, "FLAC · 24-bit 48 kHz 5.1 · lossless");
    case(8, 24, 192_000, "FLAC · 24-bit 192 kHz 7.1 · lossless");
    case(3, 16, 44100, "FLAC · 16-bit 44.1 kHz 3 channels · lossless");
  }
}
