use super::*;

#[derive(Parser)]
pub(crate) struct Manifest {
  #[arg(long = "format", default_value_t)]
  format: Format,
  #[arg(help = MANIFEST_PATH_HELP)]
  path: Option<Utf8PathBuf>,
}

impl Manifest {
  pub(crate) fn run(self) -> Result {
    let manifest = crate::Manifest::load(self.path.as_deref())?;

    match self.format {
      Format::Json => println!("{}", serde_json::to_string(&manifest).unwrap()),
      Format::JsonPretty => println!("{}", serde_json::to_string_pretty(&manifest).unwrap()),
      Format::Tsv => return Err(error::ManifestTsv.build()),
    }

    Ok(())
  }
}
