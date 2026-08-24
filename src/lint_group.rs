use super::*;

#[derive(Clone, Copy, EnumIter, Eq, Ord, PartialEq, PartialOrd, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum LintGroup {
  All,
  Compatibility,
  Content,
  Distribution,
  Junk,
}

impl LintGroup {
  #[cfg(test)]
  fn is_superset(self) -> bool {
    match self {
      Self::All | Self::Distribution => true,
      Self::Compatibility | Self::Content | Self::Junk => false,
    }
  }

  pub(crate) fn lints(self) -> BTreeSet<Lint> {
    use Lint::*;

    match self {
      Self::All => Lint::iter().collect(),
      Self::Compatibility => [
        CaseConflict,
        WindowsLeadingSpace,
        WindowsReservedCharacter,
        WindowsReservedFilename,
        WindowsTrailingPeriod,
        WindowsTrailingSpace,
      ]
      .into(),
      Self::Content => [CoverArtMissing].into(),
      Self::Distribution => &Self::Junk.lints() | &Self::Compatibility.lints(),
      Self::Junk => [Junk].into(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn all_lints_are_in_all_group() {
    let all = LintGroup::All.lints();
    for lint in Lint::iter() {
      assert!(all.contains(&lint), "lint `{lint}` not in `all` lint group");
    }
  }

  #[test]
  fn all_lints_are_in_at_least_one_group() {
    let mut lints = BTreeSet::new();
    for group in LintGroup::iter() {
      let group_lints = group.lints();

      if group.is_superset() {
        continue;
      }

      lints.extend(group_lints);
    }

    for lint in Lint::iter() {
      assert!(lints.contains(&lint), "lint `{lint}` not in group");
    }
  }
}
