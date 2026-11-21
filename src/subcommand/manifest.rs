use super::*;

#[derive(Parser)]
pub(crate) struct Manifest {
  #[arg(help = MANIFEST_PATH_HELP, value_hint = ValueHint::AnyPath)]
  path: Option<Utf8PathBuf>,
}

pub(crate) type Input = ();

pub(crate) type Output = crate::Manifest;

impl Manifest {
  pub(crate) fn run(self) -> Result {
    let manifest = crate::Manifest::load(self.path.as_deref())?;

    let count = manifest.unknown_signatures();

    ensure! {
      count == 0,
      error::UnknownSignatures { count },
    }

    DisplayJson::<Output>::print(&manifest)?;

    Ok(())
  }
}
