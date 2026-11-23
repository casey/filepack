use super::*;

#[derive(Default)]
pub(crate) struct Linter {
  active: BTreeSet<Lint>,
  allowed: BTreeSet<Lint>,
  case_conflicts: HashMap<String, Vec<RelativePath>>,
  errors: u64,
  expected: BTreeSet<Lint>,
  triggered: BTreeSet<Lint>,
}

impl Linter {
  fn active(&self) -> &BTreeSet<Lint> {
    &self.active
  }

  pub(crate) fn allow(&mut self, selectors: &[LintSelector]) {
    for selector in selectors {
      if let LintSelector::Lint(lint) = selector {
        self.allowed.insert(*lint);
      }

      for lint in selector.lints() {
        self.active.remove(&lint);
      }
    }
  }

  pub(crate) fn check(&self) -> Result {
    ensure! {
      self.errors == 0,
      error::Lint {
        count: self.errors,
      }
    }
    Ok(())
  }

  pub(crate) fn deny(&mut self, selectors: &[LintSelector]) {
    for selector in selectors {
      self.active.append(&mut selector.lints());
    }
  }

  pub(crate) fn done(mut self) -> Result {
    let unfulfilled = self
      .expected
      .difference(&self.triggered)
      .copied()
      .collect::<Vec<Lint>>();

    for lint in unfulfilled {
      self.report("unfulfilled lint expectation", lint, &[]);
    }

    self.check()
  }

  pub(crate) fn error(&mut self, lint: LintError) {
    self.error_paths(lint, &[]);
  }

  pub(crate) fn error_path(&mut self, lint: LintError, path: &RelativePath) {
    self.error_paths(lint, slice::from_ref(path));
  }

  pub(crate) fn error_paths(&mut self, error: LintError, paths: &[RelativePath]) {
    let lint = Lint::from(&error);
    if self.expected.contains(&lint) {
      self.triggered.insert(lint);
    } else {
      self.report(error, lint, paths);
    }
  }

  pub(crate) fn expect(&mut self, lints: impl IntoIterator<Item = Lint>) -> Result {
    for lint in lints {
      ensure! {
        !self.allowed.contains(&lint),
        error::LintAllowedAndExpected { lint },
      }
      self.active.insert(lint);
      self.expected.insert(lint);
    }
    Ok(())
  }

  pub(crate) fn is_active(&self, lint: Lint) -> bool {
    self.active.contains(&lint)
  }

  pub(crate) fn lint_case_conflicts(&mut self) {
    let case_conflicts = mem::take(&mut self.case_conflicts);
    for mut originals in case_conflicts.into_values() {
      if originals.len() > 1 {
        originals.sort();
        self.error_paths(LintError::CaseConflict, &originals);
      }
    }
  }

  pub(crate) fn lint_content(
    &mut self,
    root: &Utf8Path,
    options: &Options,
    metadata: Option<&Metadata>,
  ) -> Result {
    let Some(metadata) = metadata else {
      return Ok(());
    };

    if (self.is_active(Lint::AudioEmbeddedArtworkMissing)
      || self.is_active(Lint::AudioEmbeddedArtworkAspectRatio))
      && let Some(Media::Audio { items }) = &metadata.media
    {
      let failures = {
        let bar = ProgressBar::count(options.quiet, items.len().into_u64(), "files");

        let mut failures = Vec::new();

        for audio in items {
          let covers = audio.content.cover_art(root)?;

          if self.is_active(Lint::AudioEmbeddedArtworkMissing) && covers.is_empty() {
            failures.push((audio, LintError::AudioEmbeddedArtworkMissing));
          }

          if self.is_active(Lint::AudioEmbeddedArtworkAspectRatio) {
            for cover in covers {
              let dimensions = cover.dimensions().context(error::Audio {
                path: root.join(audio.path()),
              })?;

              if dimensions.width != dimensions.height {
                failures.push((
                  audio,
                  LintError::AudioEmbeddedArtworkAspectRatio { dimensions },
                ));
              }
            }
          }

          bar.inc(1);
        }

        failures
      };

      for (audio, lint) in failures {
        self.error_path(lint, audio.path());
      }
    }

    if self.is_active(Lint::VideoPlaceholderDimensionMismatch)
      && let Some(Media::Video { items }) = &metadata.media
    {
      for item in items {
        if let Some(placeholder) = &item.content.placeholder
          && let Some(video) = item.content.oriented_dimensions()
          && let placeholder = placeholder.oriented_dimensions()
          && placeholder != video
        {
          self.error_path(
            LintError::VideoPlaceholderDimensionMismatch { placeholder, video },
            item.path(),
          );
        }
      }
    }

    if self.is_active(Lint::ItemTitleMissing)
      && let Some(media) = &metadata.media
      && let Some(items) = media.items()
    {
      for item in items {
        if item.title().is_none() {
          self.error_path(LintError::ItemTitleMissing, item.path());
        }
      }
    }

    if self.is_active(Lint::MixedFormats)
      && let Some(media) = &metadata.media
      && let Some(items) = media.items()
      && let formats = items.formats()
      && formats.len() > 1
    {
      self.error(LintError::MixedFormats { formats });
    }

    Ok(())
  }

  pub(crate) fn lint_metadata(&mut self, metadata: Option<&yaml::Metadata>, generate: bool) {
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

    if self.is_active(Lint::RightsMissing)
      && metadata.is_none_or(|metadata| metadata.rights.is_none())
    {
      self.error(LintError::RightsMissing);
    }

    if self.is_active(Lint::RightsLicenseMissing)
      && let Some(rights) = metadata.and_then(|metadata| metadata.rights.as_ref())
      && rights.status == rights::Status::Licensed
      && rights.license.is_none()
    {
      self.error(LintError::RightsLicenseMissing);
    }

    if self.is_active(Lint::RightsSupportMissing)
      && let Some(metadata) = metadata
      && let Some(rights) = &metadata.rights
      && matches!(
        rights.status,
        rights::Status::Permission | rights::Status::PublicDomain,
      )
      && rights.basis.is_none()
      && rights.links.is_none()
      && metadata
        .package
        .as_ref()
        .is_none_or(|package| package.sources.is_none())
    {
      self.error(LintError::RightsSupportMissing);
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

    if self.is_active(Lint::PackageSourcesMissing)
      && metadata.is_none_or(|metadata| {
        metadata
          .package
          .as_ref()
          .is_none_or(|package| package.sources.is_none())
      })
    {
      self.error(LintError::PackageSourcesMissing);
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

    if self.is_active(Lint::DerivedAssetsMissing)
      && !generate
      && metadata.is_some_and(yaml::Metadata::has_derivable_assets)
    {
      self.error(LintError::DerivedAssetsMissing);
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
  }

  pub(crate) fn lint_path(&mut self, path: &RelativePath) {
    if let Some(lint) = path.lint(self.active()) {
      self.error_path(lint, path);
    }

    if self.is_active(Lint::CaseConflict) {
      self
        .case_conflicts
        .entry(path.as_str().to_lowercase())
        .or_default()
        .push(path.clone());
    }
  }

  pub(crate) fn new() -> Self {
    Self::default()
  }

  fn report(&mut self, message: impl Display, lint: Lint, paths: &[RelativePath]) {
    let style = Style::stderr();
    eprintln!(
      "{}: {} {}",
      "error".style(style.error()),
      message.style(style.message()),
      format_args!("[{lint}]").style(style.lint()),
    );
    for (i, path) in paths.iter().enumerate() {
      eprintln!(
        "       {}─ `{path}`",
        if i < paths.len() - 1 { '├' } else { '└' }
      );
    }
    self.errors += 1;
  }
}
