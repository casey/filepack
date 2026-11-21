use super::*;

#[derive(Parser)]
pub(crate) struct State {
  #[arg(help = MANIFEST_PATH_HELP, value_hint = ValueHint::AnyPath)]
  path: Option<Utf8PathBuf>,
}

pub(crate) type Input = ();

pub(crate) type Output = crate::State;

impl State {
  pub(crate) fn run(self) -> Result {
    let loader = Loader::load(self.path.as_deref())?;

    let state = Output::load(loader.root())?;

    DisplayJson::<Output>::print(&state)?;

    Ok(())
  }
}
