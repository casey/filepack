use super::*;

#[derive(Parser)]
pub(crate) struct Create {
  #[arg(
    help = "Overwrite manifest and generated assets if they already exist",
    long
  )]
  force: bool,
  #[command(flatten)]
  packager: Packager,
}

pub(crate) type Input = ();

pub(crate) type Output = ();

impl Create {
  pub(crate) fn run(self, options: Options) -> Result {
    self.packager.package(options, self.force)
  }
}
