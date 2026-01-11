use super::*;

#[derive(Clone, ValueEnum)]
enum Format {
  Json,
  JsonPretty,
  Tsv,
}

#[derive(Parser)]
pub(crate) struct Files {
  #[arg(long = "format", default_value = "json-pretty")]
  format: Format,
  #[arg(help = MANIFEST_PATH_HELP)]
  path: Option<Utf8PathBuf>,
}

impl Files {
  pub(crate) fn run(self) -> Result {
    let (_path, manifest) = Manifest::load(self.path.as_deref())?;

    let files = manifest.files();

    match self.format {
      Format::Json => println!("{}", serde_json::to_string(&files).unwrap(),),
      Format::JsonPretty => println!("{}", serde_json::to_string_pretty(&files).unwrap(),),
      Format::Tsv => {
        for (path, file) in files {
          println!("{path}\t{}\t{}", file.hash, file.size);
        }
      }
    }

    Ok(())
  }
}
