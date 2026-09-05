use super::*;

#[derive(Default)]
pub(crate) struct Linter {
  active: BTreeSet<Lint>,
  case_conflicts: HashMap<RelativePath, Vec<RelativePath>>,
  errors: u64,
}

impl Linter {
  fn active(&self) -> &BTreeSet<Lint> {
    &self.active
  }

  pub(crate) fn allow(&mut self, selectors: &[LintSelector]) {
    for selector in selectors {
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

    if self.is_active(Lint::VideoPlaceholderDimensions)
      && let Some(Media::Video { items }) = &metadata.media
    {
      for item in items {
        if let Some(placeholder) = &item.content.placeholder
          && let Some(video) = item.content.oriented_dimensions()
          && let placeholder = placeholder.oriented_dimensions()
          && placeholder != video
        {
          self.error_path(
            LintError::VideoPlaceholderDimensions { placeholder, video },
            item.path(),
          );
        }
      }
    }

    if self.is_active(Lint::ItemTitleMissing)
      && let Some(media) = &metadata.media
      && media.ty().has_items()
    {
      for item in media.items() {
        if item.title().is_none() {
          self.error_path(LintError::ItemTitleMissing, item.path());
        }
      }
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
  }

  pub(crate) fn lint_path(&mut self, path: &RelativePath) {
    if let Some(lint) = path.lint(self.active()) {
      self.error_path(lint, path);
    }

    if self.is_active(Lint::CaseConflict) {
      self
        .case_conflicts
        .entry(path.to_lowercase())
        .or_default()
        .push(path.clone());
    }
  }

  pub(crate) fn new() -> Self {
    Self::default()
  }
}
