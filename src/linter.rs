use super::*;

pub(crate) struct Linter {
  active: BTreeSet<Lint>,
  errors: u64,
}

impl Linter {
  pub(crate) fn active(&self) -> &BTreeSet<Lint> {
    &self.active
  }

  fn check(&self) -> Result {
    ensure! {
      self.errors == 0,
      error::Lint {
        count: self.errors,
      }
    }
    Ok(())
  }

  pub(crate) fn done(self) -> Result {
    self.check()
  }

  pub(crate) fn error(&mut self, lint: LintError) {
    eprintln!("error: {lint}");
    self.errors += 1;
  }

  pub(crate) fn error_path(&mut self, lint: LintError, path: &RelativePath) {
    eprintln!("error: path failed lint: `{path}`");
    eprintln!("       └─ {lint}");
    self.errors += 1;
  }

  pub(crate) fn error_paths(&mut self, lint: LintError, originals: &[RelativePath]) {
    eprintln!("error: {lint}");
    for (i, original) in originals.iter().enumerate() {
      eprintln!(
        "       {}─ `{original}`",
        if i < originals.len() - 1 {
          '├'
        } else {
          '└'
        }
      );
    }
    self.errors += 1;
  }

  pub(crate) fn is_active(&self, lint: Lint) -> bool {
    self.active.contains(&lint)
  }

  pub(crate) fn lint_metadata(
    &mut self,
    metadata: Option<&yaml::Metadata>,
    generate: bool,
  ) -> Result {
    if self.is_active(Lint::MetadataMissing) && metadata.is_none() {
      self.error(LintError::MetadataMissing);
    }

    if self.is_active(Lint::TitleMissing)
      && metadata.is_none_or(|metadata| metadata.title.is_none())
    {
      self.error(LintError::TitleMissing);
    }

    if self.is_active(Lint::CreatorMissing)
      && metadata.is_none_or(|metadata| metadata.creator.is_none())
    {
      self.error(LintError::CreatorMissing);
    }

    if self.is_active(Lint::TimeMissing) && metadata.is_none_or(|metadata| metadata.time.is_none())
    {
      self.error(LintError::TimeMissing);
    }

    if self.is_active(Lint::PackageMissing)
      && metadata.is_none_or(|metadata| metadata.package.is_none())
    {
      self.error(LintError::PackageMissing);
    }

    if self.is_active(Lint::PackageCreatorMissing)
      && metadata.is_none_or(|metadata| {
        metadata
          .package
          .as_ref()
          .is_none_or(|package| package.creator.is_none())
      })
    {
      self.error(LintError::PackageCreatorMissing);
    }

    if self.is_active(Lint::MediaMissing)
      && metadata.is_none_or(|metadata| metadata.media.is_none())
    {
      self.error(LintError::MediaMissing);
    }

    if self.is_active(Lint::MediaItemsMissing)
      && metadata.is_none_or(|metadata| {
        metadata
          .media
          .as_ref()
          .is_none_or(yaml::Media::items_missing)
      })
    {
      self.error(LintError::MediaItemsMissing);
    }

    if self.is_active(Lint::ArtworkMissing)
      && metadata.is_none_or(|metadata| metadata.artwork.is_none())
    {
      self.error(LintError::ArtworkMissing);
    }

    if self.is_active(Lint::NotGenerated) && !generate && metadata.is_some() {
      self.error(LintError::NotGenerated);
    }

    if self.is_active(Lint::VideoPlaceholderMissing)
      && let Some(metadata) = metadata
      && let Some(yaml::Media::Video { items }) = &metadata.media
    {
      for video in items {
        if video.placeholder.is_none() {
          self.error_path(LintError::VideoPlaceholderMissing, &video.path);
        }
      }
    }

    self.check()
  }

  pub(crate) fn new(active: BTreeSet<Lint>) -> Self {
    Self { active, errors: 0 }
  }
}
