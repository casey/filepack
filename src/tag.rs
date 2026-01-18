use super::*;

#[derive(Clone, Debug, DeserializeFromStr, PartialEq)]
pub(crate) struct Tag(String);

impl FromStr for Tag {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if re::TAG.is_match(s) {
      Ok(Self(s.into()))
    } else {
      Err(format!(
        "tags must match regex `{}`",
        &re::TAG.as_str()[1..re::TAG.as_str().len() - 1],
      ))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn invalid() {
    #[track_caller]
    fn case(s: &str) {
      assert_eq!(
        s.parse::<Tag>().unwrap_err(),
        r"tags must match regex `[0-9A-Z]+(\.[0-9A-Z]+)*`",
      );
    }

    case("");
    case("A.");
    case("A.B.");
  }

  #[test]
  fn valid() {
    #[track_caller]
    fn case(s: &str) {
      s.parse::<Tag>().unwrap();
    }

    case("A");
    case("A0");
    case("A0.A0");
    case("A0.A0.A0");
  }
}
