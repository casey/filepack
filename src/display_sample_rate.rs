use super::*;

pub(crate) struct DisplaySampleRate(pub(crate) u64);

impl Display for DisplaySampleRate {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{} kHz", DisplayMillis(self.0.into()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    #[track_caller]
    fn case(sample_rate: u64, expected: &str) {
      assert_eq!(DisplaySampleRate(sample_rate).to_string(), expected);
    }

    case(0, "0 kHz");
    case(8000, "8 kHz");
    case(22050, "22.05 kHz");
    case(44100, "44.1 kHz");
    case(44101, "44.101 kHz");
    case(48000, "48 kHz");
    case(192_000, "192 kHz");
  }
}
