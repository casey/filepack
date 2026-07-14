use super::*;

pub(crate) struct Iso8601Duration(pub(crate) Duration);

impl Display for Iso8601Duration {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let seconds = self.0.as_secs();

    let hours = seconds / 3600;
    let minutes = seconds / 60 % 60;
    let seconds = seconds % 60;

    if hours > 0 {
      write!(f, "PT{hours}H{minutes}M{seconds}S")
    } else {
      write!(f, "PT{minutes}M{seconds}S")
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    #[track_caller]
    fn case(duration: Duration, expected: &str) {
      assert_eq!(Iso8601Duration(duration).to_string(), expected);
    }

    case(Duration::ZERO, "PT0M0S");
    case(Duration::from_millis(59999), "PT0M59S");
    case(Duration::from_mins(1), "PT1M0S");
    case(Duration::from_secs(225), "PT3M45S");
    case(Duration::from_secs(3599), "PT59M59S");
    case(Duration::from_hours(1), "PT1H0M0S");
    case(Duration::from_secs(3723), "PT1H2M3S");
  }
}
