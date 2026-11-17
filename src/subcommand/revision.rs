use super::*;

#[derive(Parser)]
pub(crate) struct Revision {
  #[arg(help = MANIFEST_PATH_HELP, value_hint = ValueHint::AnyPath)]
  path: Option<Utf8PathBuf>,
}

pub(crate) type Input = ();

pub(crate) type Output = ();

impl Revision {
  pub(crate) fn run(self) -> Result {
    let loader = Loader::load(self.path.as_deref())?;

    loader.unpack()?;

    println!("{}", loader.revision_hash()?);

    Ok(())
  }
}
