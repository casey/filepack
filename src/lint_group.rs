use super::*;

#[derive(Clone, Copy, ValueEnum)]
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
        FilenameLength,
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
