use super::*;

#[derive(Parser)]
pub(crate) struct Size {
  #[arg(help = MANIFEST_PATH_HELP)]
  path: Option<Utf8PathBuf>,
}

impl Size {
  pub(crate) fn run(self) -> Result {
    let (_manifest, totals) = Loader::load(self.path.as_deref())?.unpack_with_totals()?;

    serde_json::to_writer_pretty(io::stdout(), &totals).context(error::SerializeStdout)?;

    println!();

    Ok(())
  }
}
