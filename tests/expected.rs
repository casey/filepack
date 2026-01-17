use {super::*, pretty_assertions::assert_eq};

#[derive(Debug)]
pub(crate) enum Expected {
  Empty,
  Regex(Regex),
  String(String),
}

impl Expected {
  #[track_caller]
  pub(crate) fn check(&self, actual: &str, name: &str) {
    match self {
      Expected::String(expected) => assert_eq!(actual, expected, "{name} did not match"),
      Expected::Regex(regex) => assert!(
        regex.is_match(actual),
        "{name} did not match regex\n   actual: {actual}\n    regex: {}",
        regex.as_str()
      ),
      Expected::Empty => assert!(actual.is_empty(), "{name} is not empty: {actual}"),
    }
  }

  pub(crate) fn regex(pattern: &str) -> Self {
    Self::Regex(Regex::new(&format!("^(?s){pattern}$")).unwrap())
  }

  pub(crate) fn string(string: impl Into<String>) -> Self {
    Self::String(string.into())
  }
}
