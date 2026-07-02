use super::*;

pub(crate) struct DisplayDuration(pub(crate) Duration);

impl Display for DisplayDuration {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    let seconds = self.0.as_secs();

    let hours = seconds / 3600;
    let minutes = seconds / 60 % 60;
    let seconds = seconds % 60;

    if hours > 0 {
      write!(f, "{hours}:{minutes:02}:{seconds:02}")
    } else {
      write!(f, "{minutes}:{seconds:02}")
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
      assert_eq!(DisplayDuration(duration).to_string(), expected);
    }

    case(Duration::ZERO, "0:00");
    case(Duration::from_millis(59999), "0:59");
    case(Duration::from_mins(1), "1:00");
    case(Duration::from_secs(225), "3:45");
    case(Duration::from_secs(3599), "59:59");
    case(Duration::from_hours(1), "1:00:00");
    case(Duration::from_secs(3723), "1:02:03");
  }
}
