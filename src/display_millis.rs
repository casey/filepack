use super::*;

pub(crate) struct DisplayMillis(pub(crate) u128);

impl Display for DisplayMillis {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let foo = self.0 / 1000;
    let frac = format!("{:03}", self.0 % 1000);
    let frac = frac.trim_end_matches('0');

    if frac.is_empty() {
      write!(f, "{foo}")
    } else {
      write!(f, "{foo}.{frac}")
    }
  }
}
