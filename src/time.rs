use super::*;

const EPOCH: civil::Date = civil::date(1970, 1, 1);

static TIME: LazyLock<Regex> = LazyLock::new(|| {
  r"^(?<year>0|-?[1-9][0-9]*)(-(?<month>[0-9]{2})-(?<day>[0-9]{2}))?$"
    .parse()
    .unwrap()
});

#[allow(clippy::arbitrary_source_item_ordering)]
#[derive(Clone, Debug, Decode, DeserializeFromStr, Encode, PartialEq, SerializeDisplay)]
#[cbor(validate)]
pub(crate) enum Time {
  #[n(0)]
  Year {
    #[n(0)]
    year: i64,
  },
  #[n(1)]
  Day {
    #[n(0)]
    days: i32,
  },
}

impl Time {
  fn date(days: i32) -> Result<civil::Date, TimeError> {
    jiff::Span::new()
      .try_days(days)
      .ok()
      .and_then(|span| EPOCH.checked_add(span).ok())
      .context(time_error::Days { days })
  }
}

impl Display for Time {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Day { days } => {
        let date = Self::date(*days).unwrap();
        write!(f, "{}-{:02}-{:02}", date.year(), date.month(), date.day())
      }
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

    let date = i16::try_from(year)
      .ok()
      .and_then(|year| {
        civil::Date::new(
          year,
          month.as_str().parse().unwrap(),
          captures["day"].parse().unwrap(),
        )
        .ok()
      })
      .context(time_error::Invalid { input })?;

    Ok(Self::Day {
      days: (date - EPOCH).get_days(),
    })
  }
}

impl Validate for Time {
  fn validate(&self) -> Result<(), DecodeError> {
    match self {
      Self::Day { days } => Self::date(*days).map(|_| ()),
      Self::Year { .. } => Ok(()),
    }
    .context(decode_error::Time)
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
    #[track_caller]
    fn case(value: Time, expected: TimeError) {
      assert_matches!(
        Time::decode_from_slice(&value.encode_to_vec()),
        Err(DecodeError::Time { source }) if source == expected,
      );
    }

    case(
      Time::Day { days: -4_371_588 },
      TimeError::Days { days: -4_371_588 },
    );
    case(
      Time::Day { days: 2_932_897 },
      TimeError::Days { days: 2_932_897 },
    );
    case(
      Time::Day { days: i32::MIN },
      TimeError::Days { days: i32::MIN },
    );
    case(
      Time::Day { days: i32::MAX },
      TimeError::Days { days: i32::MAX },
    );

    assert_matches!(
      Time::decode_from_slice(&hex::decode("8202a0").unwrap()),
      Err(DecodeError::InvalidDiscriminant {
        discriminant: 2,
        name: "Time",
      }),
    );
  }

  #[test]
  fn encoding() {
    #[track_caller]
    fn case(s: &str, cbor: &str) {
      assert_cbor(s.parse::<Time>().unwrap(), cbor);
    }

    case("0", "8200a10000");
    case("-44", "8200a100382b");
    case("1970", "8200a1001907b2");
    case("1970-01-01", "8201a10000");
    case("1969-12-31", "8201a10020");
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
    case("1970-00-01");
    case("1970-13-01");
    case("1970-02-30");
    case("2023-02-29");
    case("10000-01-01");
    case("-10000-01-01");
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
    case("9223372036854775807", Time::Year { year: i64::MAX });
    case("-9223372036854775808", Time::Year { year: i64::MIN });

    case("1970-01-01", Time::Day { days: 0 });
    case("1969-12-31", Time::Day { days: -1 });
    case("929-01-01", Time::Day { days: -380_217 });
    case("-44-03-15", Time::Day { days: -735_525 });
    case("0-01-01", Time::Day { days: -719_528 });
    case("2024-02-29", Time::Day { days: 19_782 });
    case("-9999-01-01", Time::Day { days: -4_371_587 });
    case("9999-12-31", Time::Day { days: 2_932_896 });
  }
}
