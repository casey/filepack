use {
  super::*,
  chrono::{FixedOffset, NaiveDate},
};

#[derive(Clone, Debug, DeserializeFromStr, PartialEq)]
pub(crate) enum DateTime {
  Date(NaiveDate),
  DateTime(chrono::DateTime<FixedOffset>),
  Year(u16),
}

impl FromStr for DateTime {
  type Err = chrono::ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if re::YEAR.is_match(s) {
      Ok(Self::Year(s.parse().unwrap()))
    } else if re::DATE.is_match(s) {
      Ok(Self::Date(s.parse()?))
    } else {
      Ok(Self::DateTime(s.parse()?))
    }
  }
}

impl Display for DateTime {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Date(date) => write!(f, "{date}"),
      Self::DateTime(date_time) => write!(f, "{date_time}"),
      Self::Year(year) => write!(f, "{year}"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dates_in_readme_are_valid() {
    let readme = filesystem::read_to_string("README.md").unwrap();

    let re = Regex::new(r"(?s)```tsv(.*?)```").unwrap();

    for capture in re.captures_iter(&readme) {
      for line in capture[1].lines() {
        if line.is_empty() {
          continue;
        }
        assert!(line.parse::<DateTime>().is_ok(), "invalid date {line}");
      }
    }
  }

  #[test]
  fn invalid() {
    #[track_caller]
    fn case(s: &str) {
      assert!(s.parse::<DateTime>().is_err());
    }

    case("1970-01-01 00:00:00");
    case("1970-01-01T00:00:00");
  }

  #[test]
  fn valid() {
    #[track_caller]
    fn case(s: &str, expected: &str) {
      assert_eq!(s.parse::<DateTime>().unwrap().to_string(), expected);
    }

    case("0", "0");
    case("00", "0");
    case("000", "0");
    case("0000", "0");
    case("1970", "1970");
    case("1970-01-01 00:00:00 +00:00", "1970-01-01 00:00:00 +00:00");
    case("1970-01-01 00:00:00Z", "1970-01-01 00:00:00 +00:00");
    case("1970-01-01", "1970-01-01");
    case("1970-01-01T00:00:00+00:00", "1970-01-01 00:00:00 +00:00");
    case("1970-01-01T00:00:00Z", "1970-01-01 00:00:00 +00:00");
    case("9999", "9999");
  }
}
