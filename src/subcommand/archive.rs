use super::*;

#[derive(Parser)]
pub(crate) struct Archive {
  #[arg(help = "Read manifest JSON from <INPUT>")]
  input: Utf8PathBuf,
  #[arg(help = "Write archive CBOR to <OUTPUT>")]
  output: Utf8PathBuf,
}

impl Archive {
  pub(crate) fn run(self) -> Result {
    let json = filesystem::read_to_string(&self.input)?;

    let manifest = Manifest::from_json(&json, &self.input)?;

    manifest.save(&self.output)?;

    Ok(())
  }
}
