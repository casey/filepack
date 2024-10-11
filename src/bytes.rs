use super::*;

pub(crate) struct Bytes(pub(crate) u128);

impl Display for Bytes {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    const DISPLAY_SUFFIXES: &[&str] = &["KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"];

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
      DISPLAY_SUFFIXES[i - 1]
    };

    let formatted = format!("{value:.2}");
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    write!(f, "{trimmed} {suffix}")
  }
}
