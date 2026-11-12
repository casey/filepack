use super::*;

#[derive(Parser)]
pub(crate) struct Metadata {
  #[arg(long = "format", default_value_t, help = FORMAT_HELP)]
  format: OutputFormatJson,
  #[arg(
    help = "Load deco metadata from <PATH>. May be path to metadata, to directory containing \
            metadata named `metadata.filemeta`, or omitted, in which case metadata named \
            `metadata.filemeta` in the current directory is loaded.",
    value_hint = ValueHint::AnyPath
  )]
  path: Option<Utf8PathBuf>,
}

pub(crate) type Input = ();

pub(crate) type Output = crate::Metadata;

impl Metadata {
  pub(crate) fn run(self) -> Result {
    let path = if let Some(path) = self.path {
      if path.is_dir() {
        path.join_case_sensitive(crate::Metadata::DECO_FILENAME)?
      } else {
        path
      }
    } else {
      Utf8Path::new(".").join_case_sensitive(crate::Metadata::DECO_FILENAME)?;
      crate::Metadata::DECO_FILENAME.into()
    };

    let bytes = filesystem::read(&path)?;

    let metadata =
      Output::decode_from_slice(&bytes).context(metadata_error::DecodeMetadataDeco { path })?;

    match self.format {
      OutputFormatJson::Json => println!("{}", serde_json::to_string(&metadata).unwrap()),
      OutputFormatJson::JsonPretty => {
        println!("{}", serde_json::to_string_pretty(&metadata).unwrap());
      }
    }

    Ok(())
  }
}
