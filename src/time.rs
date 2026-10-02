use super::*;

static TIME: LazyLock<Regex> = LazyLock::new(|| {
  r"^(?<year>0|-?[1-9][0-9]*)(-(?<month>[0-9]{2})-(?<day>[0-9]{2}))?$"
    .parse()
    .unwrap()
});

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Clone, Copy, Debug, Decode, DeserializeFromStr, Encode, PartialEq, SerializeDisplay)]
pub(crate) enum Time {
  #[n(0)]
  Year {
    #[n(1)]
    year: i64,
  },
  #[n(1)]
  Day {
    #[n(1)]
    days: i64,
  },
}

impl Time {
  pub(crate) fn year(&self) -> i64 {
    match self {
      Self::Day { days } => Date::from_days(*days).year(),
      Self::Year { year } => *year,
    }
  }
}

impl Display for Time {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Day { days } => write!(f, "{}", Date::from_days(*days)),
      Self::Year { year } => write!(f, "{year}"),
    }
  }
}

impl FromStr for Time {
  type Err = TimeError;

  fn from_str(input: &str) -> Result<Self, Self::Err> {
    let captures = TIME
      .captures(input)
      .context(time_error::Invalid { input })?;

    let year = captures["year"]
      .parse::<i64>()
      .ok()
      .context(time_error::Invalid { input })?;

    let Some(month) = captures.name("month") else {
      return Ok(Self::Year { year });
    };

    let date = Date::new(
      year,
      month.as_str().parse().unwrap(),
      captures["day"].parse().unwrap(),
    )?;

    Ok(Self::Day { days: date.days() })
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
        assert!(line.parse::<Time>().is_ok(), "invalid date {line}");
      }
    }
  }

  #[test]
  fn decode_error() {
    assert_matches!(
      Time::decode_from_slice(&[0x82, 0x02, 0x80]),
      Err(DecodeError::Unknown {
        source: UnknownError::Discriminant {
          discriminant: 2,
          name: "time",
        },
        strict: false
      }),
    );
  }

  #[test]
  fn extremes_day() {
    #[track_caller]
    fn case(days: i64, expected: &str) {
      let time = Time::Day { days };
      assert_eq!(time.to_string(), expected);
      assert_eq!(expected.parse::<Time>().unwrap(), time);
    }

    case(i64::MIN, "-25252734927764585-06-07");
    case(i64::MAX, "25252734927768524-07-27");
  }

  #[test]
  fn extremes_year() {
    #[track_caller]
    fn case(year: i64, expected: &str) {
      let time = Time::Year { year };
      assert_eq!(time.to_string(), expected);
      assert_eq!(expected.parse::<Time>().unwrap(), time);
    }

    case(i64::MIN, "-9223372036854775808");
    case(i64::MAX, "9223372036854775807");
  }

  #[test]
  fn invalid() {
    #[track_caller]
    fn case(s: &str) {
      assert_eq!(
        s.parse::<Time>().unwrap_err(),
        TimeError::Invalid { input: s.into() },
      );
    }

    case("");
    case("01");
    case("0001");
    case("-0");
    case("+1");
    case("9223372036854775808");
    case("-9223372036854775809");
    case("0929-01-01");
    case("1970-1-01");
    case("1970-01-1");
    case("1970/01/01");
    case("1970-01-01 00:00:00 +00:00");
    case("1970-01-01T00:00:00Z");
  }

  #[test]
  fn invalid_date() {
    #[track_caller]
    fn case(s: &str, expected: TimeError) {
      assert_eq!(s.parse::<Time>().unwrap_err(), expected);
    }

    case("1970-00-01", TimeError::Month { month: 0 });
    case("1970-13-01", TimeError::Month { month: 13 });
    case(
      "1970-01-00",
      TimeError::Day {
        day: 0,
        month: 1,
        year: 1970,
      },
    );
    case(
      "1970-02-30",
      TimeError::Day {
        day: 30,
        month: 2,
        year: 1970,
      },
    );
    case(
      "1900-02-29",
      TimeError::Day {
        day: 29,
        month: 2,
        year: 1900,
      },
    );
    case(
      "2000-02-30",
      TimeError::Day {
        day: 30,
        month: 2,
        year: 2000,
      },
    );
    case(
      "2023-02-29",
      TimeError::Day {
        day: 29,
        month: 2,
        year: 2023,
      },
    );
    case(
      "99999999999999999-01-01",
      TimeError::Year {
        year: 99_999_999_999_999_999,
      },
    );
    case(
      "-99999999999999999-01-01",
      TimeError::Year {
        year: -99_999_999_999_999_999,
      },
    );
    case(
      "9223372036854775807-01-01",
      TimeError::Year { year: i64::MAX },
    );
    case(
      "-9223372036854775808-01-01",
      TimeError::Year { year: i64::MIN },
    );
  }

  #[test]
  fn valid() {
    #[track_caller]
    fn case(s: &str, expected: Time) {
      let actual = s.parse::<Time>().unwrap();
      assert_eq!(actual, expected);
      assert_eq!(actual.to_string(), s);
    }

    case("0", Time::Year { year: 0 });
    case("-1", Time::Year { year: -1 });
    case("-44", Time::Year { year: -44 });
    case("1970", Time::Year { year: 1970 });
    case("10000", Time::Year { year: 10000 });
    case(
      "-13787000000",
      Time::Year {
        year: -13_787_000_000,
      },
    );

    case("1970-01-01", Time::Day { days: 0 });
    case("1969-12-31", Time::Day { days: -1 });
    case("929-01-01", Time::Day { days: -380_217 });
    case("-44-03-15", Time::Day { days: -735_525 });
    case("0-01-01", Time::Day { days: -719_528 });
    case("2000-02-29", Time::Day { days: 11_016 });
    case("2024-02-29", Time::Day { days: 19_782 });
    case("-9999-01-01", Time::Day { days: -4_371_587 });
    case("9999-12-31", Time::Day { days: 2_932_896 });
    case("10000-01-01", Time::Day { days: 2_932_897 });
    case("-10000-01-01", Time::Day { days: -4_371_953 });
    case(
      "-13787000000-01-01",
      Time::Day {
        days: -5_035_599_067_028,
      },
    );
  }

  #[test]
  fn year() {
    #[track_caller]
    fn case(s: &str, expected: i64) {
      assert_eq!(s.parse::<Time>().unwrap().year(), expected);
    }

    case("2024", 2024);
    case("2024-01-15", 2024);
    case("-44-03-15", -44);
    case("-13787000000-06-01", -13_787_000_000);
  }
}
