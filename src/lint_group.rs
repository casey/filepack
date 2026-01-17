use super::*;

#[derive(Clone, Copy, EnumIter, Eq, Ord, PartialEq, PartialOrd, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum LintGroup {
  Compatibility,
  Distribution,
  Junk,
}

impl LintGroup {
  pub(crate) fn lints(self) -> BTreeSet<Lint> {
    use Lint::*;

    match self {
      Self::Compatibility => [
        CaseConflict,
        WindowsLeadingSpace,
        WindowsReservedCharacter,
        WindowsReservedFilename,
        WindowsTrailingPeriod,
        WindowsTrailingSpace,
      ]
      .into(),
      Self::Distribution => &Self::Junk.lints() | &Self::Compatibility.lints(),
      Self::Junk => [Junk].into(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn all_lints_are_in_at_least_one_group() {
    let mut lints = BTreeSet::new();
    for group in LintGroup::iter() {
      lints.append(&mut group.lints());
    }

    for lint in Lint::iter() {
      assert!(lints.contains(&lint), "lint {lint} not in group");
    }
  }
}
