use super::*;

#[derive(Parser)]
pub(crate) struct Forget {
  #[arg(help = MANIFEST_PATH_HELP, value_hint = ValueHint::AnyPath)]
  path: Option<Utf8PathBuf>,
  #[arg(
    add = ArgValueCandidates::new(State::server_candidates),
    help = "Forget server at <URL>",
    long,
    value_name = "URL",
  )]
  server: ServerUrl,
}

pub(crate) type Input = ();

pub(crate) type Output = ();

impl Forget {
  pub(crate) fn run(self) -> Result {
    let loader = Loader::load(self.path.as_deref())?;

    let root = loader.root();

    let mut state = State::load(root)?;

    let server = self.server.as_str();

    state
      .servers
      .remove(server)
      .context(error::StateEntryMissing { server })?;

    state.save(root)?;

    Ok(())
  }
}
