use super::*;

#[derive(
  Clone, Copy, Debug, EnumIter, EnumString, Eq, IntoStaticStr, Ord, PartialEq, PartialOrd, Serialize,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum LintGroup {
  All,
  Compatibility,
  Distribution,
  Hygiene,
  Metadata,
}

impl LintGroup {
  #[cfg(test)]
  fn is_superset(self) -> bool {
    match self {
      Self::All | Self::Distribution => true,
      Self::Compatibility | Self::Hygiene | Self::Metadata => false,
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
      Self::Distribution => &Self::Hygiene.lints() | &Self::Compatibility.lints(),
      Self::Hygiene => [Junk].into(),
      Self::Metadata => [
        ArtworkMissing,
        AudioEmbeddedArtworkMissing,
        CreatorMissing,
        MediaItemsMissing,
        MediaMissing,
        MetadataMissing,
        NotGenerated,
        PackageCreatorMissing,
        PackageMissing,
        TimeMissing,
        TitleMissing,
        VideoPlaceholderDimensions,
        VideoPlaceholderMissing,
      ]
      .into(),
    }
  }

  fn name(self) -> &'static str {
    self.into()
  }
}

impl Display for LintGroup {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.name())
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
  fn all_lints_are_in_exactly_one_leaf_group() {
    for lint in Lint::iter() {
      let count = LintGroup::iter()
        .filter(|group| !group.is_superset() && group.lints().contains(&lint))
        .count();

      assert_eq!(count, 1, "lint `{lint}` in {count} leaf groups");
    }
  }

  #[test]
  fn lint_and_lint_group_names_are_disjoint() {
    let lints = Lint::iter().map(Lint::name).collect::<BTreeSet<&str>>();
    let groups = LintGroup::iter()
      .map(LintGroup::name)
      .collect::<BTreeSet<&str>>();
    assert!(lints.is_disjoint(&groups));
  }
}
