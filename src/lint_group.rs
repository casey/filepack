use super::*;

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum LintGroup {
  All,
}

impl LintGroup {
  pub(crate) fn lints(self) -> BTreeSet<Lint> {
    use Lint::*;
    [
      CaseConflict,
      FilenameLength,
      Junk,
      WindowsLeadingSpace,
      WindowsReservedCharacter,
      WindowsReservedFilename,
      WindowsTrailingPeriod,
      WindowsTrailingSpace,
    ]
    .into()
  }
}
