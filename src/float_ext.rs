pub(crate) trait FloatExt {
  fn into_u64(self) -> Option<u64>;
}

impl FloatExt for f64 {
  fn into_u64(self) -> Option<u64> {
    #![allow(
      clippy::cast_possible_truncation,
      clippy::cast_precision_loss,
      clippy::cast_sign_loss
    )]

    let converted = self as u64;

    if converted as f64 != self {
      return None;
    }

    if converted >= 1 << 53 {
      return None;
    }

    Some(converted)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn into_u64() {
    #[track_caller]
    fn case(value: f64, expected: Option<u64>) {
      assert_eq!(value.into_u64(), expected);
    }

    case(0.0, Some(0));
    case(44100.0, Some(44100));
    case(2f64.powi(53) - 1.0, Some((1 << 53) - 1));
    case(0.5, None);
    case(-1.0, None);
    case(f64::NAN, None);
    case(f64::INFINITY, None);
    case(2f64.powi(53), None);
    case(2f64.powi(64), None);
  }
}
