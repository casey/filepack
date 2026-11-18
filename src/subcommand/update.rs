use super::*;

#[derive(Parser)]
pub(crate) struct Update {
  #[command(flatten)]
  packager: Packager,
}

pub(crate) type Input = ();

pub(crate) type Output = ();

impl Update {
  pub(crate) fn run(self, options: Options) -> Result {
    self.packager.package(options, packager::Operation::Update)
  }
}
