use super::*;

#[derive(Parser)]
pub(crate) struct Metadata {
  #[arg(long = "format", default_value_t)]
  format: OutputFormat,
  #[arg(
    help = "Load deco metadata from <PATH>. May be path to metadata, to directory containing \
            metadata named `metadata.filemeta`, or omitted, in which case metadata named \
            `metadata.filemeta` in the current directory is loaded."
  )]
  path: Option<Utf8PathBuf>,
}

impl Metadata {
  pub(crate) fn run(self) -> Result {
    let path = if let Some(path) = self.path {
      if path.is_dir() {
        path.join(crate::Metadata::DECO_FILENAME)
      } else {
        path
      }
    } else {
      crate::Metadata::DECO_FILENAME.into()
    };

    let bytes = filesystem::read(&path)?;

    let metadata =
      crate::Metadata::decode_from_slice(&bytes).context(error::DecodeMetadataDeco { path })?;

    match self.format {
      OutputFormat::Json => println!("{}", serde_json::to_string(&metadata).unwrap()),
      OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&metadata).unwrap()),
      OutputFormat::Tsv => return Err(error::MetadataTsv.build()),
    }

    Ok(())
  }
}
