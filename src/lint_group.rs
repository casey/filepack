use super::*;

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum LintGroup {
  Compatibility,
  Junk,
  Recommended,
}

impl LintGroup {
  pub(crate) fn lints(self) -> BTreeSet<Lint> {
    use Lint::*;

    match self {
      Self::Compatibility => [
        CaseConflict,
        FilenameLength,
        WindowsLeadingSpace,
        WindowsReservedCharacter,
        WindowsReservedFilename,
        WindowsTrailingPeriod,
        WindowsTrailingSpace,
      ]
      .into(),
      Self::Junk => [Junk].into(),
      Self::Recommended => &Self::Junk.lints() | &Self::Compatibility.lints(),
    }
  }
}
