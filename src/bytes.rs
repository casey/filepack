use super::*;

pub(crate) struct Bytes(pub(crate) u64);

impl Display for Bytes {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    const SUFFIXES: &[&str] = &["KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];

    #[allow(clippy::cast_precision_loss)]
    let mut value = self.0 as f64;

    let mut i = 0;

    while value >= 1024.0 {
      value /= 1024.0;
      i += 1;
    }

    let suffix = if i == 0 {
      if value == 1.0 {
        "byte"
      } else {
        "bytes"
      }
    } else {
      SUFFIXES[i - 1]
    };

    let formatted = format!("{value:.2}");
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    write!(f, "{trimmed} {suffix}")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const KI: u64 = 1 << 10;
  const MI: u64 = KI << 10;
  const GI: u64 = MI << 10;
  const TI: u64 = GI << 10;
  const PI: u64 = TI << 10;
  const EI: u64 = PI << 10;

  #[test]
  fn display() {
    #[track_caller]
    fn case(bytes: u64, expected: &str) {
      assert_eq!(Bytes(bytes).to_string(), expected);
    }

    case(0, "0 bytes");
    case(1, "1 byte");
    case(2, "2 bytes");
    case(KI, "1 KiB");
    case(512 * KI, "512 KiB");
    case(MI, "1 MiB");
    case(MI + 512 * KI, "1.5 MiB");
    case(1024 * MI + 512 * MI, "1.5 GiB");
    case(GI, "1 GiB");
    case(TI, "1 TiB");
    case(PI, "1 PiB");
    case(EI, "1 EiB");
    case(u64::MAX, "16 EiB");
  }
}
