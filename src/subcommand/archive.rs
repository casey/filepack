use {super::*, io::Read};

#[derive(Parser)]
pub(crate) struct Archive {
  #[arg(help = "Write archive to <PATH>")]
  path: Utf8PathBuf,
}

impl Archive {
  pub(crate) fn run(self) -> Result {
    let mut json = String::new();
    io::stdin()
      .read_to_string(&mut json)
      .context(error::StandardInputIo)?;

    let manifest = Manifest::from_json(&json, &self.path)?;

    manifest.save(&self.path)?;

    Ok(())
  }
}
