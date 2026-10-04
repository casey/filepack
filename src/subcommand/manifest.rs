use super::*;

#[derive(Parser)]
pub(crate) struct Manifest {
  #[arg(long = "format", default_value_t)]
  format: OutputFormat,
  #[arg(help = MANIFEST_PATH_HELP)]
  path: Option<Utf8PathBuf>,
}

impl Manifest {
  pub(crate) fn run(self) -> Result {
    let manifest = crate::Manifest::load(self.path.as_deref())?;

    let count = manifest.unknown_signatures();

    ensure! {
      count == 0,
      error::UnknownSignatures { count },
    }

    match self.format {
      OutputFormat::Json => println!("{}", serde_json::to_string(&manifest).unwrap()),
      OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&manifest).unwrap()),
      OutputFormat::Tsv => return Err(error::ManifestTsv.build()),
    }

    Ok(())
  }
}
