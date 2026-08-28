use super::*;

pub(crate) struct Linter {
  active: BTreeSet<Lint>,
  errors: u64,
}

impl Linter {
  pub(crate) fn active(&self) -> &BTreeSet<Lint> {
    &self.active
  }

  pub(crate) fn done(self) -> Result {
    ensure! {
      self.errors == 0,
      error::Lint {
        count: self.errors,
      }
    }
    Ok(())
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

  pub(crate) fn new(active: BTreeSet<Lint>) -> Self {
    Self { active, errors: 0 }
  }
}
