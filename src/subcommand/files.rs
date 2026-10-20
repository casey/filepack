use super::*;

#[derive(Parser)]
pub(crate) struct Files {
  #[arg(long = "format", default_value_t)]
  format: OutputFormat,
  #[arg(help = MANIFEST_PATH_HELP, value_hint = ValueHint::AnyPath)]
  path: Option<Utf8PathBuf>,
}

pub(crate) type Input = ();

pub(crate) type Output = BTreeMap<RelativePath, File>;

impl Files {
  pub(crate) fn run(self) -> Result {
    let manifest = Manifest::load(self.path.as_deref())?;

    let files = manifest.files();

    match self.format {
      OutputFormat::Json => println!("{}", serde_json::to_string::<Output>(&files).unwrap()),
      OutputFormat::JsonPretty => println!(
        "{}",
        serde_json::to_string_pretty::<Output>(&files).unwrap()
      ),
      OutputFormat::Tsv => {
        for (path, file) in files {
          println!("{path}\t{}\t{}", file.hash, file.size);
        }
      }
    }

    Ok(())
  }
}
