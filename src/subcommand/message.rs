use super::*;

#[derive(Parser)]
pub(crate) struct Message {
  #[arg(help = "Write message to <PATH>", value_name = "PATH")]
  output: Utf8PathBuf,
  #[arg(help = MANIFEST_PATH_HELP)]
  path: Option<Utf8PathBuf>,
  #[arg(help = TIME_HELP, long)]
  time: bool,
}

impl Message {
  pub(crate) fn run(self) -> Result {
    let manifest = Manifest::load(self.path.as_deref())?;

    let message = manifest.message(self.time)?;

    filesystem::write(&self.output, message.serialize().as_bytes())?;

    Ok(())
  }
}
