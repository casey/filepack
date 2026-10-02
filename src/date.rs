use super::*;

const CENTURY_DAYS: i128 = 36_524;
const EPOCH_DAYS: i128 = 719_468;
const ERA_DAYS: i128 = 146_097;
const ERA_YEARS: i128 = 400;
const YEAR_DAYS: i128 = 365;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Date {
  day: u8,
  month: u8,
  year: i64,
}

impl Date {
  pub(crate) fn days(self) -> i64 {
    self.days_opt().unwrap()
  }

  fn days_opt(self) -> Option<i64> {
    let year = i128::from(self.year) - i128::from(self.month <= 2);
    let month = i128::from(self.month);
    let era = year.div_euclid(ERA_YEARS);
    let yoe = year.rem_euclid(ERA_YEARS);
    let mp = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * mp + 2) / 5 + i128::from(self.day) - 1;
    let doe = yoe * YEAR_DAYS + yoe / 4 - yoe / 100 + doy;
    i64::try_from(era * ERA_DAYS + doe - EPOCH_DAYS).ok()
  }

  pub(crate) fn from_days(days: i64) -> Self {
    let z = i128::from(days) + EPOCH_DAYS;
    let era = z.div_euclid(ERA_DAYS);
    let doe = z.rem_euclid(ERA_DAYS);
    let yoe = (doe - doe / (4 * YEAR_DAYS) + doe / CENTURY_DAYS - doe / (ERA_DAYS - 1)) / YEAR_DAYS;
    let doy = doe - (YEAR_DAYS * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = u8::try_from(doy - (153 * mp + 2) / 5 + 1).unwrap();
    let month = u8::try_from(if mp < 10 { mp + 3 } else { mp - 9 }).unwrap();
    let year = yoe + era * ERA_YEARS + i128::from(month <= 2);
    Self {
      day,
      month,
      year: i64::try_from(year).unwrap(),
    }
  }

  pub(crate) fn new(year: i64, month: u8, day: u8) -> Result<Self, TimeError> {
    let days_in_month = match month {
      1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
      4 | 6 | 9 | 11 => 30,
      2 => {
        if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
          29
        } else {
          28
        }
      }
      _ => return Err(TimeError::Month { month }),
    };

    ensure!(
      (1..=days_in_month).contains(&day),
      time_error::Day { day, month, year }
    );

    let date = Self { day, month, year };

    ensure!(date.days_opt().is_some(), time_error::Year { year });

    Ok(date)
  }

  pub(crate) fn year(self) -> i64 {
    self.year
  }
}

impl Display for Date {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}-{:02}-{:02}", self.year, self.month, self.day)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn boundaries() {
    assert_eq!(
      Date::new(25_252_734_927_768_524, 7, 27).unwrap().days(),
      i64::MAX
    );
    assert_eq!(
      Date::new(25_252_734_927_768_524, 7, 28),
      Err(TimeError::Year {
        year: 25_252_734_927_768_524
      })
    );
    assert_eq!(
      Date::new(-25_252_734_927_764_585, 6, 7).unwrap().days(),
      i64::MIN
    );
    assert_eq!(
      Date::new(-25_252_734_927_764_585, 6, 6),
      Err(TimeError::Year {
        year: -25_252_734_927_764_585
      })
    );
  }

  #[test]
  fn sweep() {
    fn successor(date: Date) -> Date {
      let leap = date.year % 4 == 0 && (date.year % 100 != 0 || date.year % 400 == 0);
      let days_in_month = match date.month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
      };
      if date.day < days_in_month {
        Date {
          day: date.day + 1,
          ..date
        }
      } else if date.month < 12 {
        Date {
          day: 1,
          month: date.month + 1,
          ..date
        }
      } else {
        Date {
          day: 1,
          month: 1,
          year: date.year + 1,
        }
      }
    }

    let start = i64::try_from(-EPOCH_DAYS - ERA_DAYS / 2).unwrap();
    let end = i64::try_from(-EPOCH_DAYS + ERA_DAYS / 2).unwrap();

    let mut date = Date::from_days(start);

    for days in start..=end {
      assert_eq!(Date::from_days(days), date, "{days}");
      assert_eq!(date.days(), days, "{date}");
      date = successor(date);
    }
  }
}
