use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum LintSelector {
  Group(LintGroup),
  Lint(Lint),
}

impl LintSelector {
  pub(crate) fn candidates() -> Vec<CompletionCandidate> {
    let mut registry = json::Registry::new();

    Lint::json_schema(&mut registry);
    LintGroup::json_schema(&mut registry);

    registry
      .values()
      .flat_map(|schema| {
        let json::Schema::StringEnum { values, .. } = schema else {
          unreachable!()
        };

        values
          .iter()
          .map(|value| (value.value.clone(), value.description.clone()))
      })
      .collect::<BTreeMap<String, String>>()
      .into_iter()
      .map(|(name, description)| CompletionCandidate::new(name).help(Some(description.into())))
      .collect()
  }

  pub(crate) fn lints(self) -> BTreeSet<Lint> {
    match self {
      Self::Group(group) => group.lints(),
      Self::Lint(lint) => [lint].into(),
    }
  }
}

impl FromStr for LintSelector {
  type Err = String;

  fn from_str(name: &str) -> Result<Self, Self::Err> {
    if let Ok(lint) = name.parse() {
      Ok(Self::Lint(lint))
    } else if let Ok(group) = name.parse() {
      Ok(Self::Group(group))
    } else {
      Err(format!("unknown lint or lint group `{name}`"))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parsing() {
    #[track_caller]
    fn case(name: &str, expected: Result<LintSelector, &str>) {
      assert_eq!(
        name.parse::<LintSelector>(),
        expected.map_err(str::to_string),
      );
    }

    case("junk", Ok(LintSelector::Lint(Lint::Junk)));
    case(
      "distribution",
      Ok(LintSelector::Group(LintGroup::Distribution)),
    );
    case("foo", Err("unknown lint or lint group `foo`"));
  }
}
