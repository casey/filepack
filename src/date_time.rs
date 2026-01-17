use {
  super::*,
  chrono::{FixedOffset, NaiveDate},
};

#[derive(Clone, Debug, DeserializeFromStr, PartialEq)]
pub(crate) enum DateTime {
  Date(NaiveDate),
  DateTime(chrono::DateTime<FixedOffset>),
}

impl FromStr for DateTime {
  type Err = chrono::ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if re::DATE.is_match(s) {
      Ok(Self::Date(s.parse()?))
    } else {
      Ok(Self::DateTime(s.parse()?))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn date_time_formats() {
    #[track_caller]
    fn case(s: &str) {
      assert_eq!(
        serde_json::from_str::<DateTime>(&format!("\"{s}\"")).unwrap(),
        s.parse::<DateTime>().unwrap(),
      );
    }

    case("1970-01-01");
    case("1970-01-01T00:00:00Z");
    case("1970-01-01T00:00:00+00:00");
  }
}
