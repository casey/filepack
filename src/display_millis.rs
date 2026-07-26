use super::*;

pub(crate) struct DisplayMillis(pub(crate) u128);

impl Display for DisplayMillis {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let integer = self.0 / 1000;
    let fractional = format!("{:03}", self.0 % 1000);
    let fractional = fractional.trim_end_matches('0');
    if fractional.is_empty() {
      write!(f, "{integer}")
    } else {
      write!(f, "{integer}.{fractional}")
    }
  }
}
